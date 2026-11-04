mod document;
mod header;
mod markdown;
mod media;
mod page;
mod topic;

use std::fmt;

pub use document::{DocumentFacts, PreparedDocument, ReaderOptions, prepare_document};
pub use header::{HeaderField, SourceRange};
pub use markdown::{Analysis, RenderNode, RenderedBody};
pub use media::{ImageInfo, LocalMedia, MediaFolder, MediaSource, inspect_image};
pub use topic::Topic;

pub fn render_shell(
    topic: &Topic,
    body_html: &str,
    lang: &str,
    extra_css: &str,
) -> Result<String, PageError> {
    page::render_shell(
        topic,
        body_html,
        markdown::code_styles_css(),
        lang,
        extra_css,
    )
}

pub const fn writing_guide() -> &'static str {
    include_str!("../skill/notion-sayfa/references/yazim.md")
}

/// A parsed page whose source can be rendered without parsing again.
pub struct PreparedPage {
    topic: topic::Topic,
    markdown: markdown::PreparedMarkdown,
}

#[derive(Debug)]
pub enum PageError {
    Document(String),
    Media(String),
}

impl fmt::Display for PageError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document(message) | Self::Media(message) => output.write_str(message),
        }
    }
}

impl std::error::Error for PageError {}

/// Parse a page without accessing files or media.
pub fn prepare_page(source: &str) -> Result<PreparedPage, PageError> {
    let (topic, body) = topic::Topic::parse(source)?;
    let markdown = markdown::prepare(body);
    Ok(PreparedPage { topic, markdown })
}

/// Render a prepared page using media supplied by the caller.
pub fn render_page(page: &PreparedPage, media: &impl MediaSource) -> Result<String, PageError> {
    let body = markdown::render(&page.markdown, media)?;
    page::render(&page.topic, &body, markdown::code_styles_css())
}
