use std::ops::Range;

use markdown_it::Node;
use markdown_it::parser::inline::{Text, TextSpecial};
use markdown_it::plugins::cmark::block::code::CodeBlock;
use markdown_it::plugins::cmark::block::fence::CodeFence;
use markdown_it::plugins::cmark::block::heading::ATXHeading;
use markdown_it::plugins::cmark::block::lheading::SetextHeader;
use markdown_it::plugins::cmark::block::list::ListItem;
use markdown_it::plugins::cmark::block::paragraph::Paragraph;
use markdown_it::plugins::cmark::block::reference::Definition;
use markdown_it::plugins::cmark::inline::autolink::Autolink;
use markdown_it::plugins::cmark::inline::backticks::CodeInline;
use markdown_it::plugins::cmark::inline::image::Image;
use markdown_it::plugins::cmark::inline::link::Link;
use markdown_it::plugins::cmark::inline::newline::{Hardbreak, Softbreak};
use markdown_it::plugins::extra::tables::TableCell;
use markdown_it::plugins::html::{html_block::HtmlBlock, html_inline::HtmlInline};
use markdown_it_footnote::collect::FootnotesContainerNode;
use markdown_it_footnote::inline::InlineFootnote;

use crate::PageError;
use crate::markdown::blocks::{Container, Summary};
use crate::markdown::footnotes::UnusedDefinitions;

use super::blocks::{is_named_note, notes, summary_span};
use super::source::Source;
use super::tracking::{CodeBytes, DestinationSpan};
use super::{Analysis, CodeSignature, ContainerTitle, ContainerUse, Destination, ProtectedRange};

pub(super) fn read(
    root: &Node,
    source: &Source<'_>,
    output: &mut Analysis,
) -> Result<(), PageError> {
    let (named_notes, inline_notes): (Vec<_>, Vec<_>) =
        notes(root).partition(|note| is_named_note(note));
    let mut queries = Queries {
        source,
        output,
        inline_notes,
        text: Vec::new(),
        signatures: Vec::new(),
    };
    queries.visit(root, false)?;
    for note in named_notes {
        queries.visit(note, false)?;
    }
    queries.finish();
    Ok(())
}

struct Queries<'a, 'document> {
    source: &'a Source<'document>,
    output: &'a mut Analysis,
    inline_notes: Vec<&'a Node>,
    text: Vec<TextPart>,
    signatures: Vec<(usize, CodeSignature)>,
}

struct TextPart {
    offset: usize,
    text: String,
    title: bool,
}

impl<'a> Queries<'a, '_> {
    fn visit(&mut self, node: &Node, image_alt: bool) -> Result<(), PageError> {
        if node.is::<FootnotesContainerNode>() || node.is::<UnusedDefinitions>() {
            return Ok(());
        }
        self.record_structure(node)?;
        self.record_text(node)?;
        self.record_protected(node)?;
        self.record_images(node, image_alt)?;
        for child in &node.children {
            self.visit(child, image_alt || node.is::<Image>())?;
        }
        if let Some(note) = self.inline_note(node) {
            self.visit_inline_note(note)?;
        }
        Ok(())
    }

    fn record_structure(&mut self, node: &Node) -> Result<(), PageError> {
        if let Some(container) = node.cast::<Container>() {
            let span = self.source.container_span(node)?;
            self.output.container_uses.push(ContainerUse {
                name: container.name(),
                source_range: self.source.range(span)?,
            });
        }
        if node.is::<HtmlBlock>() || node.is::<HtmlInline>() {
            let mut span = self.source.node(node)?;
            if node.is::<HtmlBlock>() {
                span = self.source.with_line_end(span);
            }
            self.output
                .formatting
                .raw_html_ranges
                .push(self.source.range(span)?);
        }
        Ok(())
    }

    fn record_text(&mut self, node: &Node) -> Result<(), PageError> {
        if let Some((kind, language, content)) = code(node) {
            return self.record_code(node, kind, language, content);
        }
        if node.is::<Summary>() {
            return self.record_title(node);
        }
        if is_inline_owner(node) {
            return self.record_inline_group(node, &node.children);
        }
        if node.is::<ListItem>() {
            self.record_tight_item(&node.children)?;
        }
        Ok(())
    }

    fn record_code(
        &mut self,
        node: &Node,
        kind: &'static str,
        language: &str,
        content: &str,
    ) -> Result<(), PageError> {
        let start = self.source.node(node)?.start;
        self.add_text(start, content.trim_end_matches('\n').to_owned(), false);
        self.signatures.push((
            start,
            CodeSignature {
                kind,
                language: language.into(),
                content_bytes: node
                    .ext
                    .get::<CodeBytes>()
                    .map_or_else(|| content.as_bytes().to_vec(), |bytes| bytes.0.clone()),
            },
        ));
        Ok(())
    }

    fn record_title(&mut self, node: &Node) -> Result<(), PageError> {
        if let Some(span) = summary_span(node, self.source)? {
            let text = self.inline_text(&node.children);
            self.add_text(span.start, text.clone(), true);
            self.output
                .formatting
                .container_titles
                .push(ContainerTitle {
                    text,
                    source_range: self.source.range(span)?,
                });
        }
        Ok(())
    }

    fn record_inline_group(&mut self, owner: &Node, nodes: &[Node]) -> Result<(), PageError> {
        let text = self.inline_text(nodes);
        if !text.is_empty() {
            let fallback = nodes.iter().find_map(|child| child.srcmap);
            if let Some(span) = owner.srcmap.or(fallback) {
                self.add_text(span.get_byte_offsets().0, text, false);
            }
        }
        Ok(())
    }

    fn record_tight_item(&mut self, children: &[Node]) -> Result<(), PageError> {
        for group in children.split(is_block) {
            if let Some(first) = group.iter().find(|node| node.srcmap.is_some()) {
                self.record_inline_group(first, group)?;
            }
        }
        Ok(())
    }

    fn add_text(&mut self, offset: usize, text: String, title: bool) {
        if !text.is_empty() {
            self.text.push(TextPart {
                offset,
                text,
                title,
            });
        }
    }

    fn inline_text(&self, nodes: &[Node]) -> String {
        let mut text = String::new();
        for node in nodes {
            self.append_inline(node, &mut text);
        }
        text
    }

    fn append_inline(&self, node: &Node, text: &mut String) {
        if let Some(value) = node.cast::<Text>() {
            text.push_str(&value.content);
        } else if let Some(value) = node.cast::<TextSpecial>() {
            text.push_str(&value.content);
        } else if node.is::<Softbreak>() || node.is::<Hardbreak>() {
            text.push('\n');
        } else if let Some(note) = self.inline_note(node) {
            text.push_str(&self.inline_text(&note.children));
        } else {
            for child in &node.children {
                self.append_inline(child, text);
            }
        }
    }

    fn inline_note(&self, node: &Node) -> Option<&'a Node> {
        if !node.is::<InlineFootnote>() {
            return None;
        }
        let (start, end) = node.srcmap?.get_byte_offsets();
        self.inline_notes.iter().copied().find(|note| {
            note.srcmap.is_some_and(|map| {
                let (left, right) = map.get_byte_offsets();
                start <= left && right <= end
            })
        })
    }

    fn visit_inline_note(&mut self, note: &Node) -> Result<(), PageError> {
        for paragraph in &note.children {
            for child in &paragraph.children {
                self.visit(child, false)?;
            }
        }
        Ok(())
    }

    fn record_protected(&mut self, node: &Node) -> Result<(), PageError> {
        if node.is::<TableCell>() {
            self.record_table_escapes(node)?;
        }
        if let Some((kind, span)) = protected(node, self.source)? {
            self.output.protected_ranges.push(ProtectedRange {
                kind,
                source_range: self.source.range(span)?,
            });
        }
        Ok(())
    }

    fn record_table_escapes(&mut self, node: &Node) -> Result<(), PageError> {
        let Some(_) = node.srcmap else { return Ok(()) };
        let span = self.source.node(node)?;
        for (offset, _) in self.source.raw(span.clone())?.match_indices(r"\|") {
            let start = span.start + offset;
            self.output.protected_ranges.push(ProtectedRange {
                kind: "escape",
                source_range: self.source.range(start..start + 2)?,
            });
        }
        Ok(())
    }

    fn record_images(&mut self, node: &Node, image_alt: bool) -> Result<(), PageError> {
        if let Some(image) = node.cast::<Image>().filter(|_| !image_alt) {
            let span = self.source.node(node)?;
            self.output.image_destinations.push(Destination {
                destination: image.url.clone(),
                source_range: self.source.range(span)?,
            });
        }
        Ok(())
    }

    fn finish(mut self) {
        self.signatures.sort_by_key(|(offset, _)| *offset);
        self.output.formatting.code_signatures = self
            .signatures
            .into_iter()
            .map(|(_, signature)| signature)
            .collect();
        self.text.sort_by_key(|item| item.offset);
        self.output.plain_text = join_text(self.text.iter());
        self.output.formatting.content_text =
            join_text(self.text.iter().filter(|item| !item.title));
        self.output
            .image_destinations
            .sort_by_key(|item| item.source_range);
        self.output
            .container_uses
            .sort_by_key(|item| item.source_range);
        self.output.formatting.raw_html_ranges.sort_unstable();
        self.output.formatting.raw_html_ranges.dedup();
        self.output
            .formatting
            .container_titles
            .sort_by_key(|item| item.source_range);
        disjoint_protected_ranges(&mut self.output.protected_ranges);
    }
}

fn join_text<'a>(text: impl Iterator<Item = &'a TextPart>) -> String {
    text.map(|item| item.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_inline_owner(node: &Node) -> bool {
    node.is::<Paragraph>()
        || node.is::<ATXHeading>()
        || node.is::<SetextHeader>()
        || node.is::<TableCell>()
}

fn is_block(node: &Node) -> bool {
    super::blocks::block_kind(node).is_some()
}

fn code(node: &Node) -> Option<(&'static str, &str, &str)> {
    if let Some(fence) = node.cast::<CodeFence>() {
        return Some(("fence", &fence.info, &fence.content));
    }
    node.cast::<CodeBlock>()
        .map(|block| ("code_block", "", block.content.as_str()))
}

fn protected(
    node: &Node,
    source: &Source<'_>,
) -> Result<Option<(&'static str, Range<usize>)>, PageError> {
    if let Some(span) = node.ext.get::<DestinationSpan>() {
        return Ok(Some((destination_kind(node), span.0.clone())));
    }
    if node.is::<Autolink>() {
        let span = source.node(node)?;
        return Ok(Some(("link_destination", span.start + 1..span.end - 1)));
    }
    if node.is::<CodeInline>() {
        return Ok(Some(("inline_code", source.node(node)?)));
    }
    escape_span(node, source).map(|span| span.map(|span| ("escape", span)))
}

fn destination_kind(node: &Node) -> &'static str {
    if node.is::<Image>() {
        "image_destination"
    } else if node.is::<Definition>() {
        "reference_destination"
    } else {
        debug_assert!(node.is::<Link>());
        "link_destination"
    }
}

fn escape_span(node: &Node, source: &Source<'_>) -> Result<Option<Range<usize>>, PageError> {
    if node
        .cast::<TextSpecial>()
        .is_some_and(|text| text.info == "escape" && text.content != text.markup)
    {
        return source.node(node).map(Some);
    }
    if node.is::<Hardbreak>()
        && let Ok(span) = source.node(node)
    {
        let raw = source.raw(span.clone())?;
        if raw.starts_with('\\') {
            return Ok(Some(span));
        }
    }
    Ok(None)
}

fn disjoint_protected_ranges(ranges: &mut Vec<ProtectedRange>) {
    ranges.sort_by_key(|range| {
        (
            range.source_range.start,
            std::cmp::Reverse(range.source_range.end),
        )
    });
    let mut end = 0;
    ranges.retain(|range| {
        if range.source_range.end <= end {
            return false;
        }
        end = range.source_range.end;
        true
    });
}
