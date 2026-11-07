mod analysis;
mod blocks;
mod footnotes;
mod highlight;
mod inline;
mod renderer;
pub(crate) mod terms;

use std::sync::LazyLock;

use markdown_it::{MarkdownIt, Node, plugins};

use crate::{MediaSource, PageError, ReaderOptions};

pub use analysis::Analysis;
pub use renderer::{RenderNode, RenderedBody};

pub(crate) use highlight::code_styles_css;

static PARSER: LazyLock<MarkdownIt> = LazyLock::new(|| make_parser(false));
static READER_PARSER: LazyLock<MarkdownIt> = LazyLock::new(|| make_parser(true));

pub(crate) struct PreparedMarkdown(Node);

pub(crate) fn prepare(source: &str) -> PreparedMarkdown {
    PreparedMarkdown(PARSER.parse(source))
}

#[cfg(test)]
pub(crate) fn render(
    prepared: &PreparedMarkdown,
    media: &(impl MediaSource + ?Sized),
) -> Result<String, PageError> {
    renderer::render(&prepared.0, media)
}

pub(crate) fn render_with_warnings(
    prepared: &PreparedMarkdown,
    media: &(impl MediaSource + ?Sized),
) -> Result<RenderedBody, PageError> {
    renderer::render_with_warnings(&prepared.0, media)
}

pub(crate) fn prepare_reader(source: &str) -> PreparedMarkdown {
    PreparedMarkdown(READER_PARSER.parse(source))
}

pub(crate) fn analyze(
    prepared: &PreparedMarkdown,
    source: &str,
    body_start: usize,
) -> Result<Analysis, PageError> {
    terms::validate(&prepared.0, source, body_start)?;
    blocks::validate(&prepared.0, source, body_start)?;
    analysis::analyze(&prepared.0, source, body_start)
}

pub(crate) fn validate_terms(
    prepared: &PreparedMarkdown,
    source: &str,
    body_start: usize,
) -> Result<(), PageError> {
    terms::validate(&prepared.0, source, body_start)
}

pub(crate) fn has_terms(prepared: &PreparedMarkdown) -> bool {
    !terms::definitions(&prepared.0).is_empty()
}

pub(crate) fn render_document(
    prepared: &PreparedMarkdown,
    media: &(impl MediaSource + ?Sized),
    options: &ReaderOptions,
) -> Result<RenderedBody, PageError> {
    renderer::render_document(&prepared.0, media, options)
}

fn make_parser(reader: bool) -> MarkdownIt {
    let mut parser = MarkdownIt::new();
    plugins::cmark::add(&mut parser);
    plugins::html::add(&mut parser);
    plugins::extra::strikethrough::add(&mut parser);
    plugins::extra::tables::add(&mut parser);
    if reader {
        blocks::add_reader(&mut parser);
        footnotes::add_reader(&mut parser);
        analysis::configure(&mut parser);
    } else {
        blocks::add(&mut parser);
        footnotes::add(&mut parser);
    }
    inline::add(&mut parser);
    terms::add(&mut parser);
    parser
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoMedia;

    impl MediaSource for NoMedia {
        fn video_data_uri(&self, _: &str) -> Result<String, PageError> {
            panic!("The document has no video")
        }

        fn image_data_uri(&self, _: &str) -> Result<String, PageError> {
            panic!("The document has no poster")
        }
    }

    #[test]
    fn nested_blocks_share_reference_context_and_ignore_code_fences() {
        let source = concat!(
            "::::callout\r\n:::toggle **Aç** [bağ][ref]\r\n",
            "[ileri][ref] [^not] ==*vurgulu*== `a==b==`\r\n\r\n",
            "```text\r\n:::\r\n::::\r\n```\r\n:::\r\n::::\r\n\r\n",
            "[^not]: Dipnot\r\n\r\n[ref]: https://example.test\r\n"
        );
        let prepared = prepare(source);
        let html = render(&prepared, &NoMedia).unwrap();

        assert!(html.contains(
            "<summary><strong>Aç</strong> <a href=\"https://example.test\">bağ</a></summary>"
        ));
        assert!(html.contains("<a href=\"https://example.test\">ileri</a>"));
        assert!(html.contains("<code>a==b==</code>"));
        assert!(html.contains("<mark><em>vurgulu</em></mark>"));
        assert!(html.contains("<code class=\"language-text\">:::\n::::\n</code>"));
        assert!(html.contains("<section class=\"footnotes\">"));
        assert!(html.contains("<li id=\"fn1\""));
    }

    #[test]
    fn parser_ranges_point_to_original_utf8_and_crlf() {
        use markdown_it::plugins::cmark::inline::backticks::CodeInline;

        let source = "## Türkçe 🐈\r\n\r\n:::toggle Ş\r\n`ö == ü`\r\n:::\r\n";
        let document = prepare(source);

        let mut code_range = None;
        document.0.walk(|node, _| {
            if node.is::<CodeInline>() {
                code_range = node.srcmap.map(|position| position.get_byte_offsets());
            }
        });
        let (start, end) = code_range.unwrap();
        assert_eq!(&source[start..end], "`ö == ü`");
    }

    #[test]
    fn tight_and_loose_task_lists_keep_checkbox_and_inline_formatting() {
        let tight = prepare("- [x] **kalın**\n- [ ] _italik_");
        let loose = prepare("- [x] **kalın**\n\n- [ ] _italik_");
        for document in [&tight, &loose] {
            let html = render(document, &NoMedia).unwrap();
            assert!(html.contains("<ul class=\"contains-task-list\">"));
            assert!(html.contains("checked=\"checked\" disabled=\"disabled\" type=\"checkbox\"> <strong>kalın</strong>"));
            assert!(html.contains("class=\"task-list-item-checkbox\" disabled=\"disabled\" type=\"checkbox\"> <em>italik</em>"));
        }
    }

    #[test]
    fn repeated_footnotes_keep_first_use_order_and_matching_backlinks() {
        let document = prepare("B[^b], A[^a], B[^b]\n\n[^a]: A notu\n\n[^b]: B notu");
        let html = render(&document, &NoMedia).unwrap();

        assert!(html.contains("href=\"#fn1\" id=\"fnref1:1\">[1:1]"));
        assert!(html.contains("href=\"#fnref1:1\" class=\"footnote-backref\""));
        assert!(html.contains("href=\"#fn2\" id=\"fnref2\">[2]"));
        assert!(html.find("B notu").unwrap() < html.find("A notu").unwrap());
    }
}
