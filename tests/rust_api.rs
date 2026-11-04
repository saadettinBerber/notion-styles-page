use std::cell::Cell;
use std::path::Path;

use notion_sayfa::{MediaSource, PageError, prepare_page, render_page, writing_guide};

#[test]
fn embedded_writing_guide_matches_the_canonical_document() {
    let guide =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("skill/notion-sayfa/references/yazim.md");
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
