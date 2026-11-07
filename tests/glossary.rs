use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use notion_sayfa::{
    MediaSource, PageError, ReaderOptions, RenderNode, prepare_document, prepare_page, render_page,
};
use regex::Regex;
use tempfile::tempdir;

const HEADER: &str = "---\ntitle: Glossary\nfont: system\n---\n\n";
const API_TERMS: &str = ":::terms\n- **API:** A **shared** `interface` with [details](https://example.com). API stays plain here.\n:::\n";
const BEFORE_SOURCE: &str = include_str!("fixtures/glossary-no-terms.md");
// Captured with /tmp/notion-page-before-glossary before implementing :::terms.
const BEFORE_HTML: &[u8] = include_bytes!("fixtures/glossary-no-terms.html");
const BEFORE_RAW_SOURCE: &str = include_str!("fixtures/glossary-no-terms-raw.md");
const BEFORE_RAW_HTML: &[u8] = include_bytes!("fixtures/glossary-no-terms-raw.html");

#[test]
fn documents_without_a_glossary_keep_previous_html_apart_from_reader_highlights() {
    let reader_assets = Regex::new(
        r"(?s)/\* reader-highlights:start \*/.*?/\* reader-highlights:end \*/\n\n|<!-- reader-highlights:start -->\n.*?<!-- reader-highlights:end -->\n",
    )
    .unwrap();
    for (source, expected) in [
        (BEFORE_SOURCE, BEFORE_HTML),
        (BEFORE_RAW_SOURCE, BEFORE_RAW_HTML),
    ] {
        let page = prepare_page(source).unwrap();
        let html = render_page(&page, &TestMedia).unwrap();
        assert_eq!(reader_assets.replace_all(&html, "").as_bytes(), expected);

        let directory = tempdir().unwrap();
        fs::write(directory.path().join("plain.md"), source).unwrap();
        let result = run(directory.path(), &["plain.md"]);
        assert_success(&result);
        assert!(result.stderr.is_empty());
        let html = fs::read_to_string(directory.path().join("plain.html")).unwrap();
        assert_eq!(reader_assets.replace_all(&html, "").as_bytes(), expected);
    }
}

#[test]
fn one_hidden_glossary_works_before_between_or_after_the_prose() {
    let pieces = [
        format!("{API_TERMS}\n## First\n\nAPI API.\n\nLast paragraph.\n"),
        format!("## First\n\nAPI API.\n\n{API_TERMS}\nLast paragraph.\n"),
        format!("## First\n\nAPI API.\n\nLast paragraph.\n\n{API_TERMS}"),
    ];
    let rendered: Vec<_> = pieces.iter().map(|body| page_html(body)).collect();
    for html in &rendered {
        assert_eq!(buttons(html), vec![("term-api".into(), "API".into())]);
        assert!(!html.contains(":::terms"));
        assert!(!html.contains("<li>"));
        assert!(html.contains("<strong>shared</strong>"));
        assert!(html.contains("<code>interface</code>"));
        assert!(html.contains(r#"href="https://example.com""#));
        assert!(html.contains("API stays plain here."));
        assert!(html.find("Last paragraph.").unwrap() < card_start(html, "term-api"));
    }
    assert!(rendered.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn first_occurrence_resets_at_h2_but_not_at_paragraphs_or_h3() {
    let html = page_html(&format!(
        "API API.\n\nAPI again.\n\n## First\n\nAPI and API.\n\n### Child\n\nAPI again.\n\n## Second\n\nAPI again.\n\n{API_TERMS}"
    ));
    assert_eq!(buttons(&html).len(), 3);
    assert!(
        buttons(&html)
            .iter()
            .all(|(target, label)| target == "term-api" && label == "API")
    );
    assert_eq!(card_ids(&html), vec!["term-api"]);
}

#[test]
fn paragraphs_lists_tables_quotes_callouts_and_toggle_bodies_are_prose() {
    let cases = [
        "**API** in a paragraph.",
        "- API in a list.",
        "| Label |\n| --- |\n| API |",
        "> API in a quotation.",
        ":::callout 💡 Note\nAPI in a callout.\n:::",
        ":::toggle Details\nAPI in the body.\n:::",
    ];
    for prose in cases {
        let html = page_html(&format!("{prose}\n\n{API_TERMS}"));
        assert_eq!(
            buttons(&html),
            vec![("term-api".into(), "API".into())],
            "{prose}"
        );
    }
}

#[test]
fn protected_content_does_not_match_or_consume_the_first_occurrence() {
    let body = r#"## API

:::toggle API
Other body.
:::

`API` and [API](https://example.com).

```text
API
```

    API

<span>API</span>

<figure><figcaption>API</figcaption></figure>

<svg xmlns="http://www.w3.org/2000/svg"><text>API</text></svg>

:::video clip.mp4
API in a video caption.
:::

![API](image.png)

API in eligible prose.
"#;
    let html = page_html(&format!("{body}\n{API_TERMS}"));
    assert_eq!(buttons(&html), vec![("term-api".into(), "API".into())]);
    assert!(
        Regex::new(r#"<p><button\b[^>]*>API</button> in eligible prose\.</p>"#)
            .unwrap()
            .is_match(&html)
    );
    assert!(html.contains("<summary>API</summary>"));
    assert!(html.contains("<span>API</span>"));
    assert!(html.contains("<figcaption>API</figcaption>"));
    assert!(html.contains("<text>API</text>"));
    assert!(html.contains(r#"alt="API""#));
}

#[test]
fn inline_raw_html_is_skipped_and_matching_resumes_after_its_closing_tag() {
    let final_match = Regex::new(r#"and <button\b[^>]*>API</button>\."#).unwrap();
    for prose in [
        "<span>API</span> and API.",
        "<SPAN><em>API</em></SPAN> and API.",
    ] {
        let source = format!("{prose}\n\n{API_TERMS}");
        let html = page_html(&source);
        assert_eq!(buttons(&html), vec![("term-api".into(), "API".into())]);
        assert!(final_match.is_match(&html));
        for allow_raw_html in [false, true] {
            let document = prepare_document(
                &source,
                ReaderOptions {
                    allow_raw_html,
                    ..ReaderOptions::default()
                },
            )
            .unwrap();
            let rendered = document.render(&TestMedia).unwrap();
            assert_eq!(
                buttons(&rendered.html),
                vec![("term-api".into(), "API".into())]
            );
            assert_eq!(assembled(&rendered.render_nodes), rendered.html);
        }
    }
}

#[test]
fn raw_html_wrappers_keep_prose_protected_across_blank_lines() {
    let outside_match = Regex::new(r#"<button\b[^>]*>BREX</button> outside\."#).unwrap();
    for wrapper in ["figure", "svg", "div"] {
        let source = format!(
            "<{wrapper}>\n\nBREX inside.\n\nBREX in another paragraph.\n\n</{wrapper}>\n\nBREX outside.\n\n:::terms\n- **BREX:** Rules.\n:::\n"
        );
        let html = page_html(&source);
        assert_eq!(buttons(&html), vec![("term-brex".into(), "BREX".into())]);
        assert!(outside_match.is_match(&html), "page wrapper {wrapper}");
        for allow_raw_html in [false, true] {
            let document = prepare_document(
                &source,
                ReaderOptions {
                    allow_raw_html,
                    ..ReaderOptions::default()
                },
            )
            .unwrap();
            let rendered = document.render(&TestMedia).unwrap();
            assert_eq!(
                buttons(&rendered.html),
                vec![("term-brex".into(), "BREX".into())]
            );
            assert!(
                outside_match.is_match(&rendered.html),
                "reader wrapper {wrapper}, allow_raw_html={allow_raw_html}"
            );
            assert_eq!(assembled(&rendered.render_nodes), rendered.html);
        }
    }
}

#[test]
fn inline_formatting_does_not_invent_a_word_boundary() {
    let final_match = Regex::new(r#"then <button\b[^>]*>BREX</button>\."#).unwrap();
    for prefix in ["`a`", "**a**", "[a](https://example.com)", "<span>a</span>"] {
        let source = format!("{prefix}BREX then BREX.\n\n:::terms\n- **BREX:** Rules.\n:::\n");
        let html = page_html(&source);
        assert_eq!(buttons(&html), vec![("term-brex".into(), "BREX".into())]);
        assert!(final_match.is_match(&html), "prefix {prefix}");
        let document = prepare_document(&source, ReaderOptions::default()).unwrap();
        let rendered = document.render(&TestMedia).unwrap();
        assert!(
            final_match.is_match(&rendered.html),
            "reader prefix {prefix}"
        );
    }
}

#[test]
fn turkish_suffixes_are_inside_the_button_and_case_variants_are_restricted() {
    let glossary = ":::terms\n- **İşlem:** Operation.\n- **Işık:** Light.\n- **BREX:** Rules.\n- **XSD:** Schema.\n- **Veri modülü:** A module.\n- **API:** Interface.\n:::\n";
    let html = page_html(&format!(
        "## First\n\nİşlem'in Işık'tan BREX’e XSD’nin veri modülünü API'nin.\n\n## Second\n\nİŞLEM IŞIK api xİşlem işlemden ışıklar APIler.\n\n{glossary}"
    ));
    let labels: Vec<_> = buttons(&html).into_iter().map(|(_, label)| label).collect();
    assert_eq!(
        labels,
        [
            "İşlem'in",
            "Işık'tan",
            "BREX’e",
            "XSD’nin",
            "veri modülünü",
            "API'nin",
            "işlemden",
            "ışıklar",
            "APIler"
        ]
    );
}

#[test]
fn entity_and_emphasis_boundaries_keep_the_entire_suffixed_word_in_one_button() {
    for prose in [
        "BREX&#39;e and BREX.",
        "**BREX**'e and BREX.",
        "B**REX**'e and BREX.",
    ] {
        let source = format!("{prose}\n\n:::terms\n- **BREX:** Rules.\n:::\n");
        for html in page_and_reader_html(&source) {
            assert_eq!(
                buttons(&html),
                vec![("term-brex".into(), "BREX'e".into())],
                "{prose}"
            );
            let paragraph = first_paragraph(&html);
            assert_eq!(visible_text(&paragraph), "BREX'e and BREX.");
            assert_inline_nesting(&paragraph);
            if prose.contains("**") {
                assert!(paragraph.contains("<strong>"), "{paragraph}");
            }
        }
    }
}

#[test]
fn multiword_terms_cross_emphasis_without_crossing_html_tags() {
    for (prose, visible) in [
        (
            "veri **modülünü** and veri modülü.",
            "veri modülünü and veri modülü.",
        ),
        (
            "**prefix veri** modülünü and veri modülü.",
            "prefix veri modülünü and veri modülü.",
        ),
        (
            "veri **modülünü suffix** and veri modülü.",
            "veri modülünü suffix and veri modülü.",
        ),
        (
            "**prefix veri** *modülünü suffix* and veri modülü.",
            "prefix veri modülünü suffix and veri modülü.",
        ),
        (
            "veri ==modülünü== and veri modülü.",
            "veri modülünü and veri modülü.",
        ),
        (
            "veri ~~modülünü~~ and veri modülü.",
            "veri modülünü and veri modülü.",
        ),
    ] {
        let source = format!("{prose}\n\n:::terms\n- **Veri modülü:** A module.\n:::\n");
        for html in page_and_reader_html(&source) {
            assert_eq!(
                buttons(&html),
                vec![("term-veri-modulu".into(), "veri modülünü".into())],
                "{prose}"
            );
            let paragraph = first_paragraph(&html);
            assert_eq!(visible_text(&paragraph), visible);
            assert!(
                ["<strong>", "<mark>", "<s>"]
                    .iter()
                    .any(|tag| paragraph.contains(tag)),
                "{paragraph}"
            );
            assert_inline_nesting(&paragraph);
        }
    }
}

#[test]
fn softbreaks_join_prose_but_protected_inline_nodes_split_it() {
    let later_match = Regex::new(r#"and <button\b[^>]*>veri modülü</button>\."#).unwrap();
    for (prose, expected) in [
        ("veri\nmodülünü and veri modülü.", "veri modülünü"),
        ("veri `modülünü` and veri modülü.", "veri modülü"),
        (
            "veri [modülünü](https://example.com) and veri modülü.",
            "veri modülü",
        ),
        ("veri <span>modülünü</span> and veri modülü.", "veri modülü"),
    ] {
        let source = format!("{prose}\n\n:::terms\n- **Veri modülü:** A module.\n:::\n");
        for html in page_and_reader_html(&source) {
            let matches = buttons(&html);
            assert_eq!(matches.len(), 1, "{prose}");
            assert_eq!(
                matches[0]
                    .1
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
                expected
            );
            if expected == "veri modülü" {
                assert!(later_match.is_match(&html), "{prose}");
            }
            assert_inline_nesting(&first_paragraph(&html));
        }
    }
}

#[test]
fn longest_overlapping_term_wins_without_consuming_the_shorter_term() {
    let html = page_html(
        "Dil modeli ve Dil modeli, ardından Dil.\n\n:::terms\n- **Dil:** Language.\n- **Dil modeli:** A language model.\n:::\n",
    );
    assert_eq!(
        buttons(&html),
        vec![
            ("term-dil-modeli".into(), "Dil modeli".into()),
            ("term-dil".into(), "Dil".into())
        ]
    );
    assert!(
        Regex::new(r#"ardından <button\b[^>]*>Dil</button>\."#)
            .unwrap()
            .is_match(&html)
    );
}

#[test]
fn longest_crossing_overlap_wins_even_when_it_starts_later() {
    let source = "alpha beta gamma delta, then alpha beta.\n\n:::terms\n- **alpha beta:** Shorter.\n- **beta gamma delta:** Longer.\n:::\n";
    let html = page_html(source);
    assert_eq!(
        buttons(&html),
        vec![
            ("term-beta-gamma-delta".into(), "beta gamma delta".into()),
            ("term-alpha-beta".into(), "alpha beta".into())
        ]
    );
    assert!(Regex::new(r#"alpha <button\b[^>]*>beta gamma delta</button>, then <button\b[^>]*>alpha beta</button>\."#).unwrap().is_match(&html));
}

#[test]
fn turkish_slugs_are_ascii_and_do_not_collide_with_headings_or_other_terms() {
    let source = "## term-sema\n\nŞema, Sema ve Ölçüm Çığlığı.\n\n:::terms\n- **Şema:** First.\n- **Sema:** Second.\n- **Ölçüm Çığlığı:** Third.\n:::\n";
    let document = prepare_document(
        source,
        ReaderOptions {
            id_prefix: "reader-".into(),
            allow_raw_html: false,
        },
    )
    .unwrap();
    let rendered = document.render(&TestMedia).unwrap();
    let matches = buttons(&rendered.html);
    assert_eq!(matches.len(), 3);
    assert!(
        matches
            .iter()
            .all(|(id, _)| id.is_ascii() && id.starts_with("reader-term-"))
    );
    assert!(
        matches
            .iter()
            .any(|(id, _)| id == "reader-term-olcum-cigligi")
    );
    assert!(matches.iter().all(|(id, _)| id != "reader-term-sema"));
    assert!(
        rendered
            .html
            .contains(r#"<h2 id="reader-term-sema">term-sema</h2>"#)
    );
    let ids = attribute_values(&rendered.html, "id");
    assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len());
    for (target, _) in matches {
        assert_eq!(ids.iter().filter(|id| **id == target).count(), 1);
    }
    assert_eq!(assembled(&rendered.render_nodes), rendered.html);
}

#[test]
fn glossary_cards_do_not_reuse_raw_heading_ids() {
    for heading in [
        r#"<h2 id="term-brex">Heading</h2>"#,
        "<h2 id=term-brex>Heading</h2>",
        r#"<h2 id="term&#45;brex">Heading</h2>"#,
    ] {
        let source = format!("{heading}\n\nBREX is used.\n\n:::terms\n- **BREX:** Rules.\n:::\n");
        let document = prepare_document(
            &source,
            ReaderOptions {
                allow_raw_html: true,
                ..ReaderOptions::default()
            },
        )
        .unwrap();
        let rendered = document.render(&TestMedia).unwrap();
        assert_eq!(assembled(&rendered.render_nodes), rendered.html);
        for html in [page_html(&source), rendered.html] {
            assert!(html.contains(heading));
            let matches = buttons(&html);
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0].1, "BREX");
            assert_ne!(matches[0].0, "term-brex");
            assert_eq!(card_ids(&html), vec![matches[0].0.clone()]);
            let ids = attribute_values(&html, "id");
            assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len());
        }
    }
}

#[test]
fn reader_rendering_preserves_source_and_is_repeatable_with_prefixed_targets() {
    let source = format!("## API\n\n[Heading](#api) and API.\n\n{API_TERMS}");
    let document = prepare_document(
        &source,
        ReaderOptions {
            id_prefix: "book-".into(),
            allow_raw_html: false,
        },
    )
    .unwrap();
    assert_eq!(document.document().source, source);
    let first = document.render(&TestMedia).unwrap();
    let second = document.render(&TestMedia).unwrap();
    assert_eq!(first, second);
    assert_eq!(assembled(&first.render_nodes), first.html);
    assert_eq!(
        buttons(&first.html),
        vec![("book-term-api".into(), "API".into())]
    );
    assert_eq!(card_ids(&first.html), vec!["book-term-api"]);
    assert!(first.html.contains(r##"href="#book-api""##));
    assert!(first.html.contains(r#"id="book-api""#));
}

#[test]
fn malformed_duplicate_and_second_blocks_report_original_source_lines() {
    let cases = [
        (":::terms\n- API: Definition.\n:::\n", 7),
        (":::terms\n- **API:**\n:::\n", 7),
        (":::terms\n- **:** Definition.\n:::\n", 7),
        (":::terms\n- **API:** One.\n- **API:** Two.\n:::\n", 8),
        (
            ":::terms\n- **API:** One.\n:::\n\n:::terms\n- **Other:** Two.\n:::\n",
            10,
        ),
        (":::terms\n- **API:** One.\n", 6),
    ];
    for (body, line) in cases {
        for source in [
            format!("{HEADER}{body}"),
            format!("{HEADER}{body}").replace('\n', "\r\n"),
        ] {
            let page_error = match prepare_page(&source) {
                Ok(_) => panic!("accepted invalid page: {source}"),
                Err(error) => error.to_string(),
            };
            assert_line(&page_error, line);
            let reader_error = match prepare_document(&source, ReaderOptions::default()) {
                Ok(_) => panic!("accepted invalid reader document: {source}"),
                Err(error) => error.to_string(),
            };
            assert_line(&reader_error, line);
        }
    }
}

#[test]
fn cli_reports_invalid_glossary_without_writing_output() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("invalid.md"),
        format!("{HEADER}:::terms\n- API: Wrong.\n:::\n"),
    )
    .unwrap();
    let result = run(directory.path(), &["invalid.md"]);
    assert_eq!(result.status.code(), Some(1));
    assert_line(&String::from_utf8_lossy(&result.stderr), 7);
    assert!(!directory.path().join("invalid.html").exists());
}

#[test]
fn unused_terms_warn_on_stderr_without_failing_conversion() {
    let directory = tempdir().unwrap();
    let source = format!(
        "{HEADER}## Dormant\n\nAPI is used.\n\n:::terms\n- **API:** Interface.\n- **Dormant:** Only a heading.\n:::\n"
    );
    fs::write(directory.path().join("terms.md"), source).unwrap();
    let result = run(directory.path(), &["terms.md"]);
    assert_success(&result);
    let warning = String::from_utf8_lossy(&result.stderr);
    assert!(warning.contains("Dormant"), "{warning}");
    assert!(
        Regex::new("(?i)warning|uyarı|unused|kullanılma")
            .unwrap()
            .is_match(&warning),
        "{warning}"
    );
    assert!(!String::from_utf8_lossy(&result.stdout).contains("Dormant"));
    assert!(directory.path().join("terms.html").exists());
}

#[test]
fn glossary_archive_restores_every_original_byte() {
    let directory = tempdir().unwrap();
    let source = format!("{HEADER}API’nin açıklaması.\n\n{API_TERMS}\n").replace('\n', "\r\n");
    fs::write(directory.path().join("terms.md"), source.as_bytes()).unwrap();
    let result = run(directory.path(), &["--archive", "terms.md"]);
    assert_success(&result);
    assert!(!directory.path().join("terms.md").exists());
    let archived = fs::read(directory.path().join("terms.html")).unwrap();
    assert_eq!(
        buttons(std::str::from_utf8(&archived).unwrap()),
        vec![("term-api".into(), "API’nin".into())]
    );
    let restored = run(directory.path(), &["--restore", "terms.html"]);
    assert_success(&restored);
    assert_eq!(
        fs::read(directory.path().join("terms.md")).unwrap(),
        source.as_bytes()
    );
    assert_eq!(
        fs::read(directory.path().join("terms.html")).unwrap(),
        archived
    );
}

fn page_html(body: &str) -> String {
    let page = prepare_page(&format!("{HEADER}{body}")).unwrap();
    render_page(&page, &TestMedia).unwrap()
}

fn page_and_reader_html(source: &str) -> [String; 2] {
    let document = prepare_document(source, ReaderOptions::default()).unwrap();
    let rendered = document.render(&TestMedia).unwrap();
    assert_eq!(assembled(&rendered.render_nodes), rendered.html);
    [page_html(source), rendered.html]
}

fn first_paragraph(html: &str) -> String {
    Regex::new(r"(?s)<p>(.*?)</p>")
        .unwrap()
        .captures(html)
        .unwrap()[1]
        .to_owned()
}

fn visible_text(html: &str) -> String {
    Regex::new(r"<[^>]*>")
        .unwrap()
        .replace_all(html, "")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn assert_inline_nesting(html: &str) {
    let mut stack = Vec::new();
    for tag in Regex::new(r"<(\/?)(strong|em|mark|s|button)\b[^>]*>")
        .unwrap()
        .captures_iter(html)
    {
        let name = &tag[2];
        if &tag[1] == "/" {
            assert_eq!(stack.pop().as_deref(), Some(name), "misnested HTML: {html}");
        } else {
            if name == "button" {
                assert!(
                    !stack.iter().any(|open| open == "button"),
                    "nested buttons: {html}"
                );
            }
            stack.push(name.to_owned());
        }
    }
    assert!(stack.is_empty(), "unclosed inline HTML: {html}");
}

fn buttons(html: &str) -> Vec<(String, String)> {
    Regex::new(r#"(?s)<button\b([^>]*)>(.*?)</button>"#)
        .unwrap()
        .captures_iter(html)
        .filter_map(|capture| {
            if !attribute_values(&capture[1], "class")
                .iter()
                .any(|classes| classes.split_whitespace().any(|class| class == "term"))
            {
                return None;
            }
            let target = attribute_values(&capture[1], "popovertarget")
                .pop()
                .expect("term buttons target a popover");
            Some((target, visible_text(&capture[2])))
        })
        .collect()
}

fn card_ids(html: &str) -> Vec<String> {
    let popover = Regex::new(r#"\spopover(?:\s|=|>)"#).unwrap();
    Regex::new(r#"<[^>]+>"#)
        .unwrap()
        .find_iter(html)
        .filter(|tag| popover.is_match(tag.as_str()))
        .filter_map(|tag| attribute_values(tag.as_str(), "id").pop())
        .collect()
}

fn card_start(html: &str, id: &str) -> usize {
    let pattern = format!(r#"<[^>]+\bid="{}"[^>]*>"#, regex::escape(id));
    Regex::new(&pattern).unwrap().find(html).unwrap().start()
}

fn attribute_values(html: &str, name: &str) -> Vec<String> {
    Regex::new(&format!(r#"\b{}="([^"]*)""#, regex::escape(name)))
        .unwrap()
        .captures_iter(html)
        .map(|capture| capture[1].to_owned())
        .collect()
}

fn assembled(nodes: &[RenderNode]) -> String {
    nodes
        .iter()
        .map(|node| match node {
            RenderNode::Leaf { html, .. } => html.clone(),
            RenderNode::Branch {
                opening_html,
                children,
                closing_html,
                ..
            } => format!("{opening_html}{}{closing_html}", assembled(children)),
        })
        .collect()
}

fn assert_line(error: &str, line: usize) {
    assert!(
        Regex::new(&format!(r"(?i)(?:satır|line)\s*:?\s*{line}\b"))
            .unwrap()
            .is_match(error),
        "expected source line {line}: {error}"
    );
}

fn run(directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_notion-page"))
        .current_dir(directory)
        .args(arguments)
        .output()
        .unwrap()
}

fn assert_success(result: &Output) {
    assert!(
        result.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

struct TestMedia;

impl MediaSource for TestMedia {
    fn video_data_uri(&self, path: &str) -> Result<String, PageError> {
        assert_eq!(path, "clip.mp4");
        Ok("data:video/mp4;base64,AA==".into())
    }

    fn image_data_uri(&self, path: &str) -> Result<String, PageError> {
        panic!("unexpected media read: {path}")
    }
}
