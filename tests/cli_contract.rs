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
    "/skill/notion-page/templates/showcase.md"
);
const SHOWCASE_MARKUP: &[&str] = &[
    r#"<html lang="tr">"#,
    r#"<h1 class="page-title">Template Showcase</h1>"#,
    r#"class="cover cover-purple""#,
    r#">template</span>"#,
    r#"<h1 id="heading-1">Heading 1</h1>"#,
    "<mark>highlighted text</mark>",
    r#"<details class="toggle"><summary>Toggle inside a callout</summary>"#,
    r#"<details class="toggle toggle-heading">"#,
    r#"<div class="columns">"#,
    r#"<div class="diagram-scroll"><svg"#,
    "<table>",
    r#"<pre><code class="language-python"><span"#,
    r#"<pre><code class="language-text">:::example"#,
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
    assert!(help_text.contains("--font <PRESET>"));
    assert!(help_text.contains("daily-dose"));
    assert!(help_text.contains("system"));
    assert!(help_text.contains("--skills"));
    assert!(help_text.contains("--skill [<NAME>]"));
    assert!(help_text.contains("List skills and guides"));
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
fn skills_are_listed_with_descriptions_outside_the_checkout() {
    let workspace = tempdir().unwrap();
    let result = run(workspace.path(), &["--skill"]);

    assert_eq!(result.status.code(), Some(0));
    assert!(result.stderr.is_empty());
    let listing = str::from_utf8(&result.stdout).unwrap();
    let entries: Vec<_> = listing
        .lines()
        .map(|line| {
            let (name, description) = line.split_once('\t').unwrap();
            assert!(!description.is_empty());
            assert!(description.ends_with('.'));
            (name, description)
        })
        .collect();
    assert_eq!(
        entries.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
        [
            "notion-page",
            "simple",
            "turkish-writing",
            "writing",
            "drawing",
            "showcase",
            "reader"
        ]
    );
    assert_eq!(
        entries
            .iter()
            .find(|(name, _)| *name == "simple")
            .unwrap()
            .1,
        "Communicate clearly in conversations and documents with context-sensitive Markdown."
    );
    for arguments in [&["--skill", "list"][..], &["--skills"][..]] {
        assert_success(&run(workspace.path(), arguments), listing);
    }
    assert!(fs::read_dir(workspace.path()).unwrap().next().is_none());
}

#[test]
fn skill_commands_print_complete_documents_outside_the_checkout() {
    let workspace = tempdir().unwrap();

    for (name, expected) in [
        ("notion-page", include_str!("../skill/notion-page/SKILL.md")),
        ("simple", include_str!("../skill/simple/SKILL.md")),
        (
            "turkish-writing",
            include_str!("../skill/turkish-writing/SKILL.md"),
        ),
        (
            "writing",
            include_str!("../skill/notion-page/references/writing.md"),
        ),
        (
            "drawing",
            include_str!("../skill/notion-page/references/drawing.md"),
        ),
        (
            "showcase",
            include_str!("../skill/notion-page/templates/showcase.md"),
        ),
        (
            "reader",
            include_str!("../skill/notion-page/references/reader.md"),
        ),
    ] {
        assert_success(&run(workspace.path(), &["--skill", name]), expected);
    }
    assert!(fs::read_dir(workspace.path()).unwrap().next().is_none());
}

#[test]
fn legacy_guide_names_resolve_to_the_current_documents() {
    let workspace = tempdir().unwrap();

    for (legacy, current) in [
        ("communication", "simple"),
        ("basic-make-notes", "simple"),
        ("readable-instructions", "simple"),
        ("yazim", "writing"),
        ("cizim", "drawing"),
        ("vitrin", "showcase"),
        ("okuyucu", "reader"),
    ] {
        let expected = run(workspace.path(), &["--skill", current]);
        let result = run(workspace.path(), &["--skill", legacy]);

        assert_eq!(expected.status.code(), Some(0));
        assert_success(&result, str::from_utf8(&expected.stdout).unwrap());
    }
    assert!(fs::read_dir(workspace.path()).unwrap().next().is_none());
}

#[test]
fn skill_commands_reject_unknown_names_and_conversion_arguments() {
    let workspace = tempdir().unwrap();

    for arguments in [
        &["--skill", "unknown"][..],
        &["--skill", "notion-page", "notes.md"],
        &["--skill", "--output", "pages"],
        &["--skill", "--font", "system"],
        &["--skill", "--archive"],
        &["--skill", "--restore"],
        &["--skill", "--skills"],
        &["--skills", "notes.md"],
        &["--skills", "--output", "pages"],
        &["--skills", "--font", "system"],
        &["--skills", "--archive"],
        &["--skills", "--restore"],
    ] {
        let result = run(workspace.path(), arguments);
        assert_eq!(result.status.code(), Some(2), "{arguments:?}");
        assert!(result.stdout.is_empty(), "{arguments:?}");
        assert!(!result.stderr.is_empty(), "{arguments:?}");
    }
    assert!(fs::read_dir(workspace.path()).unwrap().next().is_none());
}

#[test]
fn default_output_sits_next_to_unicode_source() {
    let workspace = tempdir().unwrap();
    write(workspace.path(), "notlar/İlk sayfa.md", PAGE);

    let result = run(workspace.path(), &["notlar/İlk sayfa.md"]);

    assert_success(&result, "✓ İlk sayfa.md → İlk sayfa.html\n");
    let html = fs::read_to_string(workspace.path().join("notlar/İlk sayfa.html")).unwrap();
    assert!(html.contains("<p>Gövde</p>"));
    assert_eq!(
        fs::read(workspace.path().join("notlar/İlk sayfa.md")).unwrap(),
        PAGE.as_bytes()
    );
}

#[test]
fn font_selection_uses_cli_then_frontmatter_then_the_shared_default() {
    let workspace = tempdir().unwrap();
    write(workspace.path(), "default.md", PAGE);
    write(
        workspace.path(),
        "system.md",
        &PAGE.replace("baslik: Deneme", "baslik: Deneme\nfont: system"),
    );
    write(
        workspace.path(),
        "daily.md",
        &PAGE.replace("baslik: Deneme", "baslik: Deneme\nfont: daily-dose"),
    );

    for (arguments, output, embedded) in [
        (&["default.md"][..], "default.html", true),
        (&["default.md", "--font", "system"], "default.html", false),
        (&["system.md"], "system.html", false),
        (&["system.md", "--font", "daily-dose"], "system.html", true),
        (&["daily.md", "--font", "system"], "daily.html", false),
    ] {
        let result = run(workspace.path(), arguments);

        assert_eq!(result.status.code(), Some(0), "{arguments:?}: {result:?}");
        assert!(result.stderr.is_empty(), "{arguments:?}");
        let html = fs::read_to_string(workspace.path().join(output)).unwrap();
        assert_eq!(html.contains("@font-face"), embedded, "{arguments:?}");
        assert_eq!(
            html.contains("data:font/woff2;base64,"),
            embedded,
            "{arguments:?}"
        );
        if embedded {
            assert!(html.contains("Noto Serif"));
            assert!(html.contains("Domine"));
        }
        assert!(html.contains("<p>Gövde</p>"));
    }
}

#[test]
fn invalid_font_arguments_and_frontmatter_do_not_write_html() {
    let workspace = tempdir().unwrap();
    write(workspace.path(), "valid.md", PAGE);
    let invalid = PAGE.replace("baslik: Deneme", "baslik: Deneme\nfont: papyrus");
    write(workspace.path(), "invalid.md", &invalid);

    for arguments in [
        &["valid.md", "--font", "papyrus"][..],
        &["--restore", "valid.html", "--font", "system"],
    ] {
        let result = run(workspace.path(), arguments);

        assert_eq!(result.status.code(), Some(2), "{arguments:?}");
        assert!(result.stdout.is_empty(), "{arguments:?}");
        assert!(!result.stderr.is_empty(), "{arguments:?}");
    }

    let result = run(workspace.path(), &["invalid.md", "--font", "system"]);

    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(str::from_utf8(&result.stderr).unwrap().contains("papyrus"));
    assert!(!workspace.path().join("valid.html").exists());
    assert!(!workspace.path().join("invalid.html").exists());
    assert_eq!(
        fs::read_to_string(workspace.path().join("valid.md")).unwrap(),
        PAGE
    );
    assert_eq!(
        fs::read_to_string(workspace.path().join("invalid.md")).unwrap(),
        invalid
    );
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

    assert_success(&result, "✓ showcase.md → showcase.html\n");
    let html = fs::read_to_string(workspace.path().join("cikti/showcase.html")).unwrap();
    for markup in SHOWCASE_MARKUP {
        assert!(html.contains(markup), "missing {markup}");
    }
}

#[test]
fn output_flag_alias_produces_the_same_html() {
    let workspace = tempdir().unwrap();
    write(workspace.path(), "not.md", PAGE);

    let current = run(
        workspace.path(),
        &["not.md", "--output", "current", "--font", "system"],
    );
    let alias = run(
        workspace.path(),
        &["not.md", "--cikti", "alias", "--font", "system"],
    );

    assert_success(&current, "✓ not.md → not.html\n");
    assert_success(&alias, "✓ not.md → not.html\n");
    assert_eq!(
        fs::read(workspace.path().join("current/not.html")).unwrap(),
        fs::read(workspace.path().join("alias/not.html")).unwrap(),
    );
}

#[test]
fn archive_removes_markdown_and_restore_recovers_its_exact_bytes() {
    let workspace = tempdir().unwrap();
    let directory = workspace.path();
    let source = "---\r\nbaslik: Arşiv\r\nfont: daily-dose\r\n---\r\n\r\n__Gövde__\r\n\r\n`--> </script> &`  ";
    write(directory, "notlar/İlk sayfa.md", source);

    let archived = run(
        directory,
        &[
            "--archive",
            "notlar/İlk sayfa.md",
            "--output",
            "pages",
            "--font",
            "system",
        ],
    );

    assert_success(&archived, "✓ İlk sayfa.md → İlk sayfa.html\n");
    assert!(!directory.join("notlar/İlk sayfa.md").exists());
    let html = fs::read_to_string(directory.join("pages/İlk sayfa.html")).unwrap();
    assert!(html.contains("<p><strong>Gövde</strong></p>"));
    assert!(!html.contains("@font-face"));

    let restored = run(
        directory,
        &["--restore", "pages/İlk sayfa.html", "--output", "notlar"],
    );

    assert_success(&restored, "✓ İlk sayfa.html → İlk sayfa.md\n");
    assert_eq!(
        fs::read(directory.join("notlar/İlk sayfa.md")).unwrap(),
        source.as_bytes()
    );
    assert_eq!(
        fs::read_to_string(directory.join("pages/İlk sayfa.html")).unwrap(),
        html
    );
}

#[test]
fn archive_keeps_sources_when_a_page_fails_to_render_or_output_exists() {
    let workspace = tempdir().unwrap();
    let directory = workspace.path();
    let invalid_cover = "---\nbaslik: Deneme\nkapak: lacivert\n---\n";
    write(directory, "invalid.md", invalid_cover);
    write(directory, "existing.md", PAGE);
    write(directory, "existing.html", "Existing HTML");
    write(directory, "good.md", PAGE);

    let result = run(
        directory,
        &["--archive", "invalid.md", "existing.md", "good.md"],
    );

    assert_eq!(result.status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(directory.join("invalid.md")).unwrap(),
        invalid_cover
    );
    assert!(!directory.join("invalid.html").exists());
    assert_eq!(
        fs::read_to_string(directory.join("existing.md")).unwrap(),
        PAGE
    );
    assert_eq!(
        fs::read_to_string(directory.join("existing.html")).unwrap(),
        "Existing HTML"
    );
    assert!(!directory.join("good.md").exists());
    assert!(directory.join("good.html").is_file());
}

#[test]
fn regular_conversion_preserves_an_archive_with_the_same_output_name() {
    let workspace = tempdir().unwrap();
    let directory = workspace.path();
    let source = format!("{PAGE}\n{}", "Gövde ".repeat(2_000));
    write(directory, "original/notes.md", &source);
    write(directory, "other/notes.md", PAGE);
    let archived = run(
        directory,
        &["--archive", "original/notes.md", "--output", "shared"],
    );
    assert_success(&archived, "✓ notes.md → notes.html\n");
    let archive_path = directory.join("shared/notes.html");
    let mut archive = fs::read_to_string(&archive_path).unwrap();
    archive.push_str(&"\r\n".repeat(5_000));
    fs::write(&archive_path, &archive).unwrap();

    let result = run(directory, &["other/notes.md", "--output", "shared"]);

    assert_eq!(result.status.code(), Some(1));
    assert_eq!(fs::read_to_string(&archive_path).unwrap(), archive);
    let restored = run(directory, &["--restore", "shared/notes.html"]);
    assert_success(&restored, "✓ notes.html → notes.md\n");
    assert_eq!(
        fs::read_to_string(directory.join("shared/notes.md")).unwrap(),
        source
    );
}

#[test]
fn restore_rejects_older_html_and_preserves_an_existing_markdown_file() {
    let workspace = tempdir().unwrap();
    let directory = workspace.path();
    write(directory, "older.html", "<!doctype html><p>Older HTML</p>");
    write(directory, "notes.md", PAGE);
    let archived = run(directory, &["--archive", "notes.md"]);
    assert_success(&archived, "✓ notes.md → notes.html\n");
    write(directory, "notes.md", "Existing Markdown");

    let result = run(directory, &["--restore", "older.html", "notes.html"]);

    assert_eq!(result.status.code(), Some(1));
    let stderr = str::from_utf8(&result.stderr).unwrap();
    assert!(stderr.contains("older.html"));
    assert!(stderr.contains("notes.html"));
    assert!(!directory.join("older.md").exists());
    assert_eq!(
        fs::read_to_string(directory.join("notes.md")).unwrap(),
        "Existing Markdown"
    );
}

#[test]
fn restore_rejects_a_damaged_embedded_source() {
    let workspace = tempdir().unwrap();
    let directory = workspace.path();
    write(directory, "notes.md", PAGE);
    let archived = run(directory, &["--archive", "notes.md"]);
    assert_success(&archived, "✓ notes.md → notes.html\n");

    let path = directory.join("notes.html");
    let html = fs::read_to_string(&path).unwrap();
    let source_prefix = Regex::new(r"(<!-- notion-sayfa-markdown:v1:[[:xdigit:]]+:)L").unwrap();
    let damaged = source_prefix.replace(&html, "${1}A");
    assert_ne!(damaged, html);
    fs::write(path, damaged.as_bytes()).unwrap();

    let result = run(directory, &["--restore", "notes.html"]);

    assert_eq!(result.status.code(), Some(1));
    assert!(!directory.join("notes.md").exists());
}

#[cfg(unix)]
#[test]
fn archive_refuses_to_remove_a_symlink() {
    let workspace = tempdir().unwrap();
    let directory = workspace.path();
    write(directory, "target.md", PAGE);
    let link = directory.join("link.md");
    std::os::unix::fs::symlink("target.md", &link).unwrap();

    let result = run(directory, &["--archive", "link.md"]);

    assert_eq!(result.status.code(), Some(1));
    assert!(link.is_symlink());
    assert_eq!(
        fs::read_to_string(directory.join("target.md")).unwrap(),
        PAGE
    );
    assert!(!directory.join("link.html").exists());
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
