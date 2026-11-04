use std::ops::Range;

use markdown_it::Node;

use crate::PageError;

use super::SourceRange;

pub(super) struct Source<'a> {
    original: &'a str,
    body_start: usize,
    codepoint_starts: Vec<usize>,
}

impl<'a> Source<'a> {
    pub(super) fn new(original: &'a str, body_start: usize) -> Result<Self, PageError> {
        if !original.is_char_boundary(body_start) {
            return Err(position_error());
        }
        let codepoint_starts = original.char_indices().map(|(index, _)| index).collect();
        Ok(Self {
            original,
            body_start,
            codepoint_starts,
        })
    }

    pub(super) fn raw(&self, span: Range<usize>) -> Result<&'a str, PageError> {
        self.original
            .get(self.body_start + span.start..self.body_start + span.end)
            .ok_or_else(position_error)
    }

    pub(super) fn range(&self, span: Range<usize>) -> Result<SourceRange, PageError> {
        self.raw(span.clone())?;
        Ok(SourceRange {
            start: self.codepoints_before(span.start),
            end: self.codepoints_before(span.end),
        })
    }

    fn codepoints_before(&self, index: usize) -> usize {
        self.codepoint_starts
            .partition_point(|position| *position < self.body_start + index)
    }

    pub(super) fn node(&self, node: &Node) -> Result<Range<usize>, PageError> {
        let (start, end) = node.srcmap.ok_or_else(position_error)?.get_byte_offsets();
        self.raw(start..end)?;
        Ok(start..end)
    }

    pub(super) fn line_span(&self, node: &Node) -> Result<Range<usize>, PageError> {
        let span = self.node(node)?;
        let body = &self.original[self.body_start..];
        let start = body[..span.start]
            .rfind(['\r', '\n'])
            .map_or(0, |index| index + 1);
        let end = line_ending(body, span.end);
        Ok(start..end)
    }

    pub(super) fn container_span(&self, node: &Node) -> Result<Range<usize>, PageError> {
        let span = self.node(node)?;
        let line = self.line_span(node)?;
        let prefix = self.raw(line.start..span.start)?;
        let start = prefix.rfind('>').map_or(line.start, |quote| {
            let next = quote + 1;
            let space = prefix
                .as_bytes()
                .get(next)
                .is_some_and(|value| matches!(value, b' ' | b'\t'));
            line.start + next + usize::from(space)
        });
        Ok(start..line.end)
    }

    pub(super) fn with_line_end(&self, span: Range<usize>) -> Range<usize> {
        span.start..line_ending(&self.original[self.body_start..], span.end)
    }
}

pub(super) fn line_ending(source: &str, offset: usize) -> usize {
    let rest = &source[offset..];
    if rest.starts_with("\r\n") {
        return offset + 2;
    }
    if rest.starts_with(['\r', '\n']) {
        return offset + 1;
    }
    offset
}

fn position_error() -> PageError {
    PageError::Document("Ayrıştırıcı kaynak konumunu kaybetti.".into())
}
