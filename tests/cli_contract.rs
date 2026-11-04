use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use regex::Regex;
use tempfile::tempdir;

const PAGE: &str = "---\nbaslik: Deneme\n---\nGövde\n";
const VIDEO_PAGE: &str = "---
baslik: Video
kapak: ./medya/kapak.jpg
---

:::video medya/kayit.mp4 medya/poster.jpg
Video açıklaması.
:::

![Görsel](medya/gorsel.png)
";
const SHOWCASE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/skill/notion-sayfa/templates/vitrin.md"
);
const SHOWCASE_MARKUP: &[&str] = &[
    r#"<html lang="tr">"#,
    r#"<h1 class="page-title">Şablon Vitrini</h1>"#,
    r#"class="cover cover-purple""#,
    r#"class="tag bg-green">şablon</span>"#,
    r#"<h1 id="başlık-1">Başlık 1</h1>"#,
    "<mark>Vurgulu metin</mark>",
    r#"<details class="toggle"><summary>Callout içinde toggle</summary>"#,
    r#"<details class="toggle toggle-heading">"#,
    r#"<div class="columns">"#,
    r#"<div class="diagram-scroll"><svg"#,
    "<table>",
    r#"<pre><code class="language-python"><span"#,
    r#"<pre><code class="language-text">:::ornek"#,
    r#"class="footnotes""#,
    "@media (prefers-color-scheme: dark)",
];

#[test]
fn help_and_missing_input_have_distinct_exit_codes() {
    let workspace = tempdir().unwrap();

    let help = run(workspace.path(), &["--help"]);
    let missing = run(workspace.path(), &[]);

    assert_eq!(
        (Some(0), Some(2)),
        (help.status.code(), missing.status.code())
    );
    let help_text = str::from_utf8(&help.stdout).unwrap();
    assert!(help_text.contains("Usage: notion-page"));
    assert!(help_text.contains("--output"));
    assert!(help_text.contains("Markdown files to convert"));
    assert!(help_text.contains("\nExamples\n  notion-page notes.md"));
    assert!(help_text.contains("notion-page notes/*.md --output pages"));
    assert!(help_text.contains("\nExit Codes\n  0  Success"));
    assert!(help_text.contains("1  One or more files could not be converted"));
    assert!(help_text.contains("2  Invalid command-line arguments"));
    assert!(!missing.stderr.is_empty());
}

#[test]
fn version_reports_the_cargo_package_version() {
    let workspace = tempdir().unwrap();

    let result = run(workspace.path(), &["--version"]);

    assert_success(
        &result,
        concat!("notion-page ", env!("CARGO_PKG_VERSION"), "\n"),
    );
}

#[test]
fn default_output_sits_next_to_unicode_source() {
    let workspace = tempdir().unwrap();
    write(workspace.path(), "notlar/İlk sayfa.md", PAGE);

    let result = run(workspace.path(), &["notlar/İlk sayfa.md"]);

    assert_success(&result, "✓ İlk sayfa.md → İlk sayfa.html\n");
    let html = fs::read_to_string(workspace.path().join("notlar/İlk sayfa.html")).unwrap();
    assert!(html.contains("<p>Gövde</p>"));
}

#[test]
fn render_failure_and_missing_file_do_not_stop_other_inputs() {
    let workspace = tempdir().unwrap();
    write(workspace.path(), "hatalı.md", "no YAML heading");
    write(workspace.path(), "son.md", PAGE);

    let result = run(workspace.path(), &["hatalı.md", "yok.md", "son.md"]);

    assert_eq!(Some(1), result.status.code());
    assert_eq!(
        "✓ son.md → son.html\n",
        str::from_utf8(&result.stdout).unwrap()
    );
    let stderr = str::from_utf8(&result.stderr).unwrap();
    for pattern in [r"(?m)^✗ hatalı\.md: .+başlık bloğu", r"(?m)^✗ yok\.md: .+"] {
        assert!(Regex::new(pattern).unwrap().is_match(stderr), "{stderr}");
    }
    let html_files: Vec<_> = fs::read_dir(workspace.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "html")
        })
        .collect();
    assert_eq!([workspace.path().join("son.html")], html_files.as_slice());
}

#[test]
fn shared_output_overwrites_in_first_seen_folder_order() {
    let workspace = tempdir().unwrap();
    write(workspace.path(), "a/ilk.md", PAGE);
    write(
        workspace.path(),
        "b/ortak.md",
        &PAGE.replace("Deneme", "B klasörü"),
    );
    write(
        workspace.path(),
        "a/ortak.md",
        &PAGE.replace("Deneme", "A klasörü"),
    );

    let result = run(
        workspace.path(),
        &["a/ilk.md", "b/ortak.md", "a/ortak.md", "-o", "derin/cikti"],
    );

    assert_success(
        &result,
        "✓ ilk.md → ilk.html\n✓ ortak.md → ortak.html\n✓ ortak.md → ortak.html\n",
    );
    let html = fs::read_to_string(workspace.path().join("derin/cikti/ortak.html")).unwrap();
    assert!(html.contains("<title>B klasörü</title>"));
}

#[test]
fn output_directory_does_not_change_the_document_media_root() {
    let workspace = tempdir().unwrap();
    write(workspace.path(), "notlar/film.md", VIDEO_PAGE);
    write(workspace.path(), "notlar/medya/kayit.mp4", "video");
    write(workspace.path(), "notlar/medya/poster.jpg", "poster");
    write(workspace.path(), "cikti/medya/kayit.mp4", "wrong video");

    let result = run(workspace.path(), &["notlar/film.md", "--output", "cikti"]);

    assert_success(&result, "✓ film.md → film.html\n");
    let html = fs::read_to_string(workspace.path().join("cikti/film.html")).unwrap();
    for markup in [
        r#"src="data:video/mp4;base64,dmlkZW8=""#,
        r#"poster="data:image/jpeg;base64,cG9zdGVy""#,
        "url('./medya/kapak.jpg')",
        r#"src="medya/gorsel.png""#,
    ] {
        assert!(html.contains(markup), "missing {markup}");
    }
}

#[test]
fn showcase_renders_documented_blocks_with_bundled_styles() {
    let workspace = tempdir().unwrap();

    let result = run(workspace.path(), &[SHOWCASE, "-o", "cikti"]);

    assert_success(&result, "✓ vitrin.md → vitrin.html\n");
    let html = fs::read_to_string(workspace.path().join("cikti/vitrin.html")).unwrap();
    for markup in SHOWCASE_MARKUP {
        assert!(html.contains(markup), "missing {markup}");
    }
}

#[test]
fn legacy_command_and_flag_produce_the_same_html() {
    let workspace = tempdir().unwrap();
    write(workspace.path(), "not.md", PAGE);

    let current = run(workspace.path(), &["not.md", "--output", "current"]);
    let legacy = Command::new(env!("CARGO_BIN_EXE_uret"))
        .args(["not.md", "--cikti", "legacy"])
        .current_dir(workspace.path())
        .output()
        .unwrap();

    assert_success(&current, "✓ not.md → not.html\n");
    assert_success(&legacy, "✓ not.md → not.html\n");
    assert_eq!(
        fs::read(workspace.path().join("current/not.html")).unwrap(),
        fs::read(workspace.path().join("legacy/not.html")).unwrap(),
    );
}

fn run(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_notion-page"))
        .args(arguments)
        .current_dir(directory)
        .output()
        .unwrap()
}

fn write(directory: &Path, name: &str, content: &str) {
    let path = directory.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn assert_success(result: &Output, report: &str) {
    assert_eq!(
        (Some(0), report, ""),
        (
            result.status.code(),
            str::from_utf8(&result.stdout).unwrap(),
            str::from_utf8(&result.stderr).unwrap(),
        )
    );
}
