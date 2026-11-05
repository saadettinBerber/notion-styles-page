use std::cell::Cell;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use notion_sayfa::{
    FontPreset, MediaSource, PageError, PageOptions, Topic, prepare_page, render_page,
    render_page_with_options, render_shell, render_shell_with_options, writing_guide,
};
use regex::Regex;

#[test]
fn font_catalogue_roundtrips_through_public_names_and_serde() {
    for preset in FontPreset::ALL {
        assert_eq!(preset.as_str().parse::<FontPreset>().unwrap(), preset);
        assert_eq!(preset.to_string(), preset.as_str());
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(serde_json::from_str::<FontPreset>(&json).unwrap(), preset);
    }
    let error = "unknown".parse::<FontPreset>().unwrap_err().to_string();
    assert!(error.contains("daily-dose, system"));
    assert!(serde_json::from_str::<FontPreset>("\"unknown\"").is_err());
}

#[test]
fn default_page_embeds_licensed_fonts_without_remote_font_requests() {
    let page = prepare_page("---\nbaslik: Türkçe\n---\nYumuşak **kalın** ve *italik*.\n").unwrap();
    let media = TestMedia::new("");
    let html = render_page(&page, &media).unwrap();

    assert!(html.contains("data-font=\"daily-dose\""));
    assert!(html.contains("SIL OPEN FONT LICENSE"));
    assert!(html.contains("font-style: italic"));
    assert!(html.contains("U+0100-02BA"));
    assert!(!html.contains("fonts.googleapis.com"));
    assert!(!html.contains("fonts.gstatic.com"));
    let embedded = Regex::new(r"data:font/woff2;base64,([A-Za-z0-9+/=]+)").unwrap();
    let fonts = embedded
        .captures_iter(&html)
        .map(|capture| STANDARD.decode(&capture[1]).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(fonts.len(), 6);
    assert!(fonts.iter().all(|bytes| bytes.starts_with(b"wOF2")));
    assert_eq!(media.reads.get(), 0);
}

#[test]
fn page_font_selection_uses_override_then_document_then_default() {
    let media = TestMedia::new("");
    let page = prepare_page("---\nbaslik: Yazı\nfont: system\n---\nMetin\n").unwrap();
    let system = render_page(&page, &media).unwrap();
    assert!(system.contains("data-font=\"system\""));
    assert!(!system.contains("data:font/woff2"));

    let daily = render_page_with_options(
        &page,
        &media,
        &PageOptions {
            font: Some(FontPreset::DailyDose),
        },
    )
    .unwrap();
    assert!(daily.contains("data-font=\"daily-dose\""));
    assert!(daily.contains("data:font/woff2"));

    let default_page = prepare_page("---\nbaslik: Yazı\nfont: null\n---\nMetin\n").unwrap();
    let system = render_page_with_options(
        &default_page,
        &media,
        &PageOptions {
            font: Some(FontPreset::System),
        },
    )
    .unwrap();
    assert!(system.contains("data-font=\"system\""));
    assert!(!system.contains("data:font/woff2"));
}

#[test]
fn shell_supports_legacy_topic_data_and_caller_style_overrides() {
    let legacy = serde_json::json!({
        "title": "Başlık", "icon": "📄", "cover": "", "date": "", "tags": [], "summary": ""
    });
    let topic: Topic = serde_json::from_value(legacy.clone()).unwrap();
    let html = render_shell(&topic, "<p>Gövde</p>", "en", ":root { --body-size: 24px; }").unwrap();
    assert!(html.contains("<html lang=\"en\">"));
    assert!(html.contains("data-font=\"daily-dose\""));
    assert!(html.find("--body-size: 24px").unwrap() > html.find("--body-size: 20px").unwrap());

    let system = render_shell_with_options(
        &topic,
        "<p>Gövde</p>",
        "tr",
        "",
        &PageOptions {
            font: Some(FontPreset::System),
        },
    )
    .unwrap();
    assert!(system.contains("data-font=\"system\""));
    assert!(!system.contains("data:font/woff2"));

    let mut invalid = legacy;
    invalid["font"] = serde_json::json!("unknown");
    assert!(serde_json::from_value::<Topic>(invalid).is_err());
}

#[test]
fn frontmatter_rejects_invalid_font_names_and_types() {
    for value in ["unknown", "''", "false", "0", "[]", "{}"] {
        let source = format!("---\nbaslik: Yazı\nfont: {value}\n---\nMetin\n");
        assert!(prepare_page(&source).is_err(), "accepted font: {value}");
    }
}

#[test]
fn embedded_writing_guide_matches_the_canonical_document() {
    let guide =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("skill/notion-page/references/writing.md");
    assert_eq!(writing_guide(), std::fs::read_to_string(guide).unwrap());
}

#[test]
fn prepared_page_can_render_with_independent_media_sources() {
    let source = "---\nbaslik: Medya\n---\n:::video clip.mp4\nAçıklama\n:::\n";
    let page = prepare_page(source).unwrap();
    let first = TestMedia::new("Zmlyc3Q=");
    let second = TestMedia::new("c2Vjb25k");

    let first_html = render_page(&page, &first).unwrap();
    let second_html = render_page(&page, &second).unwrap();

    assert!(first_html.contains("data:video/mp4;base64,Zmlyc3Q="));
    assert!(second_html.contains("data:video/mp4;base64,c2Vjb25k"));
    assert!(!second_html.contains("Zmlyc3Q="));
    assert_eq!((first.reads.get(), second.reads.get()), (1, 1));
}

struct TestMedia {
    bytes_base64: &'static str,
    reads: Cell<u32>,
}

impl TestMedia {
    fn new(bytes_base64: &'static str) -> Self {
        Self {
            bytes_base64,
            reads: Cell::new(0),
        }
    }
}

impl MediaSource for TestMedia {
    fn video_data_uri(&self, path: &str) -> Result<String, PageError> {
        assert_eq!(path, "clip.mp4");
        self.reads.set(self.reads.get() + 1);
        Ok(format!("data:video/mp4;base64,{}", self.bytes_base64))
    }

    fn image_data_uri(&self, path: &str) -> Result<String, PageError> {
        Err(PageError::Media(format!("Beklenmeyen poster: {path}")))
    }
}
