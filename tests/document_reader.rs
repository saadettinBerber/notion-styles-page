use notion_sayfa::{ReaderOptions, prepare_document};

#[test]
fn reads_headerless_fragments() {
    let document = prepare_document("İlk paragraf\n", ReaderOptions::default()).unwrap();
    assert!(document.document().topic.is_none());
}

#[test]
fn reports_english_title_as_written() {
    let source = "\u{feff}---\r\ntitle: \"Project notes 🧠\"\r\n---\r\n";
    let document = prepare_document(source, ReaderOptions::default()).unwrap();
    let field = &document.document().header_fields[0];
    let span = &field.value_range;
    let written: String = source
        .chars()
        .skip(span.start)
        .take(span.end - span.start)
        .collect();
    assert_eq!(field.name, "title");
    assert_eq!(written, "\"Project notes 🧠\"");
    assert_eq!(field.text.as_deref(), Some("Project notes 🧠"));
}

#[test]
fn reports_header_spans_in_original_unicode_coordinates() {
    let source = "\u{feff}---\r\nbaslik: &name \"Türkçe 🧠\"\r\nöz: *name\r\n---\r\n";
    let document = prepare_document(source, ReaderOptions::default()).unwrap();
    let fields = &document.document().header_fields;
    let written = |field: usize| {
        let span = &fields[field].value_range;
        source
            .chars()
            .skip(span.start)
            .take(span.end - span.start)
            .collect::<String>()
    };
    assert_eq!(
        (written(0), written(1)),
        ("\"Türkçe 🧠\"".into(), "*name".into())
    );
    assert_eq!(fields[1].text.as_deref(), Some("Türkçe 🧠"));
}
