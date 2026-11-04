use super::*;

fn inspect(source: &str, body_start: usize) -> serde_json::Value {
    let parser = super::super::make_parser(true);
    let root = parser.parse(&source[body_start..]);
    serde_json::to_value(analyze(&root, source, body_start).unwrap()).unwrap()
}

#[test]
fn analysis_preserves_unicode_source_coordinates_after_front_matter() {
    let source = "\u{feff}---\r\nbaslik: Ş\r\n---\r\n\r\n## Ö 🐈\r\n\r\nA `ü`\r\n";
    let body_start = source.find("\r\n\r\n##").unwrap() + 2;
    let result = inspect(source, body_start);
    assert_eq!(
        slice(source, &result["blocks"][0]["source_range"]),
        "## Ö 🐈\r\n"
    );
    assert_eq!(
        slice(source, &result["protected_ranges"][0]["source_range"]),
        "`ü`"
    );
}

#[test]
fn formatting_keeps_code_bytes_and_separates_toggle_title() {
    let source = ":::toggle **Özet**\r\nİçerik\r\n\r\n```rs\r\nç\r\n```\r\n:::\r\n";
    let result = inspect(source, 0);
    assert_eq!(result["blocks"][0]["title"], "**Özet**");
    assert_eq!(result["formatting"]["container_titles"][0]["text"], "Özet");
    assert_eq!(result["formatting"]["content_text"], "İçerik\nç");
    assert_eq!(
        result["formatting"]["code_signatures"][0]["content_bytes"],
        serde_json::json!("ç\r\n".as_bytes())
    );
}

#[test]
fn link_destinations_are_distinct_from_reference_image_usages() {
    let source = "![alt][key] and [link](<a(b)> 'title')\n\n[key]: img.png\n";
    let result = inspect(source, 0);
    assert_eq!(result["image_destinations"][0]["destination"], "img.png");
    assert_eq!(
        slice(source, &result["image_destinations"][0]["source_range"]),
        "![alt][key]"
    );
    let protected: Vec<_> = result["protected_ranges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| {
            (
                value["kind"].as_str().unwrap(),
                slice(source, &value["source_range"]),
            )
        })
        .collect();
    assert_eq!(
        protected,
        [
            ("link_destination", "a(b)".to_owned()),
            ("reference_destination", "img.png".to_owned())
        ]
    );
}

fn slice(source: &str, range: &serde_json::Value) -> String {
    let start = range["start"].as_u64().unwrap() as usize;
    let end = range["end"].as_u64().unwrap() as usize;
    source.chars().skip(start).take(end - start).collect()
}

#[test]
fn container_usage_keeps_indentation_inside_a_quote() {
    let source = ">  :::video x.mp4\r\n>  caption\r\n>  :::\r\n";
    let result = inspect(source, 0);
    assert_eq!(
        slice(source, &result["container_uses"][0]["source_range"]),
        " :::video x.mp4\r\n>  caption\r\n>  :::\r\n"
    );
}

#[test]
fn nested_named_footnotes_participate_in_their_source_parent_shape() {
    let source = "A[^n]\n\n> [^n]: note\n>\n> text\n";
    let result = inspect(source, 0);
    let quote = &result["blocks"][1];
    assert_eq!(quote["shape"][0][2][0][0], "definition");
    assert_eq!(result["plain_text"], "A\nnote\ntext");
}

#[test]
fn backslash_hardbreak_protects_the_original_crlf_and_following_indent() {
    let source = "text\\\r\n two  \r\nthree";
    let result = inspect(source, 0);
    let ranges = result["protected_ranges"].as_array().unwrap();
    assert_eq!(ranges.len(), 1);
    assert_eq!(slice(source, &ranges[0]["source_range"]), "\\\r\n ");
}
