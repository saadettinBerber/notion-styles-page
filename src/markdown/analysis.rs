mod blocks;
mod queries;
mod source;
mod tracking;

#[cfg(test)]
mod tests;

use markdown_it::{MarkdownIt, Node};
use serde::Serialize;
use serde_json::Value;

use crate::PageError;

use self::source::Source;

pub use crate::SourceRange;

#[derive(Serialize)]
pub struct Block {
    pub kind: String,
    pub source_range: SourceRange,
    pub html: &'static str,
    pub shape: Value,
    pub shared: bool,
    pub children: Vec<Block>,
    pub title: String,
    pub title_range: Option<SourceRange>,
}

#[derive(Serialize)]
pub struct Destination {
    pub destination: String,
    pub source_range: SourceRange,
}

#[derive(Serialize)]
pub struct ContainerUse {
    pub name: &'static str,
    pub source_range: SourceRange,
}

#[derive(Serialize)]
pub struct GlossaryTerm {
    pub term: String,
    pub definition: String,
    pub source_range: SourceRange,
}

#[derive(Serialize)]
pub struct ProtectedRange {
    pub kind: &'static str,
    pub source_range: SourceRange,
}

#[derive(Default, Serialize)]
pub struct Formatting {
    pub content_text: String,
    pub code_signatures: Vec<CodeSignature>,
    pub raw_html_ranges: Vec<SourceRange>,
    pub container_titles: Vec<ContainerTitle>,
}

#[derive(Serialize)]
pub struct CodeSignature {
    pub kind: &'static str,
    pub language: String,
    pub content_bytes: Vec<u8>,
}

#[derive(Serialize)]
pub struct ContainerTitle {
    pub text: String,
    pub source_range: SourceRange,
}

#[derive(Default, Serialize)]
pub struct Analysis {
    pub blocks: Vec<Block>,
    pub image_destinations: Vec<Destination>,
    pub container_uses: Vec<ContainerUse>,
    pub glossary_terms: Vec<GlossaryTerm>,
    pub protected_ranges: Vec<ProtectedRange>,
    pub plain_text: String,
    pub formatting: Formatting,
}

pub(crate) fn configure(parser: &mut MarkdownIt) {
    tracking::configure(parser);
}

pub(crate) fn analyze(root: &Node, source: &str, body_start: usize) -> Result<Analysis, PageError> {
    let source = Source::new(source, body_start)?;
    let mut analysis = Analysis {
        blocks: blocks::read(root, &source)?,
        ..Analysis::default()
    };
    queries::read(root, &source, &mut analysis)?;
    Ok(analysis)
}
