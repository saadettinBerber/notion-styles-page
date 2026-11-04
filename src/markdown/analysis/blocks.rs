use markdown_it::Node;
use markdown_it::plugins::cmark::block::blockquote::Blockquote;
use markdown_it::plugins::cmark::block::code::CodeBlock;
use markdown_it::plugins::cmark::block::fence::CodeFence;
use markdown_it::plugins::cmark::block::heading::ATXHeading;
use markdown_it::plugins::cmark::block::hr::ThematicBreak;
use markdown_it::plugins::cmark::block::lheading::SetextHeader;
use markdown_it::plugins::cmark::block::list::{BulletList, ListItem, OrderedList};
use markdown_it::plugins::cmark::block::paragraph::Paragraph;
use markdown_it::plugins::cmark::block::reference::Definition;
use markdown_it::plugins::cmark::inline::image::Image;
use markdown_it::plugins::cmark::inline::newline::Softbreak;
use markdown_it::plugins::extra::tables::{Table, TableBody, TableCell, TableHead, TableRow};
use markdown_it::plugins::html::html_block::HtmlBlock;
use markdown_it_footnote::collect::FootnotesContainerNode;
use markdown_it_footnote::definitions::FootnoteDefinition;
use markdown_it_footnote::inline::InlineFootnote;
use serde_json::{Value, json};

use crate::PageError;
use crate::markdown::blocks::{Container, Summary};
use crate::markdown::footnotes;

use super::source::Source;
use super::tracking::ImageLabel;
use super::{Block, SourceRange};

pub(super) fn read(root: &Node, source: &Source<'_>) -> Result<Vec<Block>, PageError> {
    let mut definitions = Vec::new();
    for note in notes(root).filter(|node| is_named_note(node)) {
        definitions.push((source.node(note)?, note));
    }
    definitions.sort_by_key(|(span, _)| span.start);
    let reader = BlockReader {
        source,
        definitions,
    };
    reader.read_children(root)
}

struct BlockReader<'a, 'source> {
    source: &'a Source<'source>,
    definitions: Vec<(std::ops::Range<usize>, &'a Node)>,
}

pub(super) fn notes(root: &Node) -> impl Iterator<Item = &Node> {
    root.children
        .iter()
        .filter(|node| node.is::<FootnotesContainerNode>())
        .flat_map(|node| node.children.iter())
        .chain(footnotes::unused_definitions(root))
}

pub(super) fn is_named_note(node: &Node) -> bool {
    node.cast::<FootnoteDefinition>()
        .is_some_and(|note| !note.inline)
}

impl<'a> BlockReader<'a, '_> {
    fn read_children(&self, node: &'a Node) -> Result<Vec<Block>, PageError> {
        self.block_children(node)
            .into_iter()
            .map(|child| self.read_block(child))
            .collect()
    }

    fn block_children(&self, node: &'a Node) -> Vec<&'a Node> {
        let children = node
            .children
            .iter()
            .filter(|node| block_kind(node).is_some());
        let mut located: Vec<_> = children
            .map(|child| {
                let span = child.srcmap.map(|pos| {
                    let (start, end) = pos.get_byte_offsets();
                    start..end
                });
                (span, child)
            })
            .collect();
        if let Some(span) = node.srcmap {
            let (start, end) = span.get_byte_offsets();
            let direct: Vec<_> = self
                .direct_definitions(node, start..end, &located)
                .map(|(span, note)| (Some(span.clone()), *note))
                .collect();
            located.extend(direct);
        }
        if located.iter().all(|(span, _)| span.is_some()) {
            located.sort_by_key(|(span, _)| span.as_ref().map(|span| span.start));
        }
        located.into_iter().map(|(_, node)| node).collect()
    }

    fn direct_definitions<'b>(
        &'b self,
        node: &'a Node,
        parent: std::ops::Range<usize>,
        children: &'b [(Option<std::ops::Range<usize>>, &'a Node)],
    ) -> impl Iterator<Item = &'b (std::ops::Range<usize>, &'a Node)> {
        let first = self
            .definitions
            .partition_point(|(span, _)| span.start < parent.start);
        let last = self
            .definitions
            .partition_point(|(span, _)| span.start < parent.end);
        let candidates = &self.definitions[first..last];
        candidates.iter().filter(move |(span, definition)| {
            !std::ptr::eq(*definition, node)
                && contains(&parent, span)
                && !children
                    .iter()
                    .any(|(child, _)| child.as_ref().is_some_and(|child| contains(child, span)))
                && !candidates.iter().any(|(outer, owner)| {
                    !std::ptr::eq(*owner, node)
                        && span != outer
                        && contains(&parent, outer)
                        && contains(outer, span)
                })
        })
    }

    fn read_block(&self, node: &'a Node) -> Result<Block, PageError> {
        let span = self.source.line_span(node)?;
        let (title, title_range) = title(node, self.source)?;
        let children = if node.is::<Container>() {
            self.read_children(node)?
        } else {
            Vec::new()
        };
        let shape = json!([node_shape(node, self, false)?]);
        Ok(Block {
            kind: block_kind(node).unwrap_or("definition").into(),
            source_range: self.source.range(span)?,
            html: "",
            shape,
            shared: is_shared(node),
            children,
            title,
            title_range,
        })
    }
}

fn contains(outer: &std::ops::Range<usize>, inner: &std::ops::Range<usize>) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

fn title(node: &Node, source: &Source<'_>) -> Result<(String, Option<SourceRange>), PageError> {
    let Some(summary) = node.children.first().filter(|child| child.is::<Summary>()) else {
        return Ok((String::new(), None));
    };
    let Some(span) = summary_span(summary, source)? else {
        return Ok((String::new(), None));
    };
    Ok((
        source.raw(span.clone())?.to_owned(),
        Some(source.range(span)?),
    ))
}

pub(super) fn summary_span(
    node: &Node,
    source: &Source<'_>,
) -> Result<Option<std::ops::Range<usize>>, PageError> {
    if node.srcmap.is_some() {
        return source.node(node).map(Some);
    }
    Ok(None)
}

pub(super) fn block_kind(node: &Node) -> Option<&str> {
    if let Some(container) = node.cast::<Container>() {
        return Some(match container.name() {
            "callout" => "container:callout",
            "toggle" => "container:toggle",
            "toggle-heading" => "container:toggle-heading",
            "columns" => "container:columns",
            "column" => "container:column",
            "video" => "container:video",
            _ => return None,
        });
    }
    kind_tag(node, false).map(|(kind, _)| kind.trim_end_matches("_open"))
}

fn kind_tag(node: &Node, table_head: bool) -> Option<(&'static str, &'static str)> {
    let entry = if node.is::<Paragraph>() {
        ("paragraph_open", "p")
    } else if node.is::<ATXHeading>() || node.is::<SetextHeader>() {
        ("heading_open", "h1")
    } else if node.is::<BulletList>() {
        ("bullet_list_open", "ul")
    } else if node.is::<OrderedList>() {
        ("ordered_list_open", "ol")
    } else if node.is::<ListItem>() {
        ("list_item_open", "li")
    } else if node.is::<Blockquote>() {
        ("blockquote_open", "blockquote")
    } else if node.is::<Table>() {
        ("table_open", "table")
    } else if node.is::<TableHead>() {
        ("thead_open", "thead")
    } else if node.is::<TableBody>() {
        ("tbody_open", "tbody")
    } else if node.is::<TableRow>() {
        ("tr_open", "tr")
    } else if node.is::<TableCell>() && table_head {
        ("th_open", "th")
    } else if node.is::<TableCell>() {
        ("td_open", "td")
    } else if node.is::<ThematicBreak>() {
        ("hr", "hr")
    } else if node.is::<CodeFence>() {
        ("fence", "code")
    } else if node.is::<CodeBlock>() {
        ("code_block", "code")
    } else if node.is::<HtmlBlock>() {
        ("html_block", "")
    } else if node.is::<Definition>() || is_named_note(node) {
        ("definition", "")
    } else {
        return None;
    };
    Some(entry)
}

fn node_shape<'a>(
    node: &'a Node,
    reader: &BlockReader<'a, '_>,
    table_head: bool,
) -> Result<Value, PageError> {
    let children = child_shapes(node, reader, table_head)?;
    if let Some(value) = special_shape(node, reader.source)? {
        return Ok(value);
    }
    if let Some(container) = node.cast::<Container>() {
        return Ok(json!([
            format!("container_{}_open", container.name()),
            "div",
            children
        ]));
    }
    let (kind, tag) = kind_tag(node, table_head).unwrap_or(("definition", ""));
    Ok(match kind {
        "ordered_list_open" => json!([kind, node.cast::<OrderedList>().unwrap().start, children]),
        "definition" => json!([kind, children]),
        "heading_open" => json!([kind, heading_tag(node), children]),
        _ => json!([kind, tag, children]),
    })
}

fn child_shapes<'a>(
    node: &'a Node,
    reader: &BlockReader<'a, '_>,
    table_head: bool,
) -> Result<Vec<Value>, PageError> {
    let table_head = table_head || node.is::<TableHead>();
    let mut shapes = Vec::new();
    if node.is::<ListItem>() {
        let inline_first = node
            .children
            .first()
            .is_some_and(|node| block_kind(node).is_none());
        if inline_first {
            shapes.push(json!(["paragraph_open", "p", []]));
        }
    }
    for child in reader.block_children(node) {
        shapes.push(node_shape(child, reader, table_head)?);
    }
    Ok(shapes)
}

fn special_shape(node: &Node, source: &Source<'_>) -> Result<Option<Value>, PageError> {
    if let Some(fence) = node.cast::<CodeFence>() {
        return Ok(Some(json!(["fence", fence.info, raw_bytes(node, source)?])));
    }
    if node.is::<CodeBlock>() {
        return Ok(Some(json!(["code_block", "", raw_bytes(node, source)?])));
    }
    if is_image_paragraph(node) {
        let images: Vec<_> = node
            .children
            .iter()
            .filter_map(|child| {
                let image = child.cast::<Image>()?;
                let label = child
                    .ext
                    .get::<ImageLabel>()
                    .map_or("", |label| label.0.as_str());
                Some((&image.url, label))
            })
            .collect();
        return Ok(Some(json!([
            "image_paragraph",
            images,
            raw_bytes(node, source)?
        ])));
    }
    Ok(None)
}

fn raw_bytes(node: &Node, source: &Source<'_>) -> Result<Value, PageError> {
    Ok(json!({"$bytes": source.raw(source.line_span(node)?)?.as_bytes()}))
}

fn heading_tag(node: &Node) -> String {
    let level = node
        .cast::<ATXHeading>()
        .map(|heading| heading.level)
        .or_else(|| node.cast::<SetextHeader>().map(|heading| heading.level));
    format!("h{}", level.unwrap_or(1))
}

pub(super) fn is_shared(node: &Node) -> bool {
    node.is::<CodeFence>()
        || node.is::<CodeBlock>()
        || node.is::<ThematicBreak>()
        || is_image_paragraph(node)
}

fn is_image_paragraph(node: &Node) -> bool {
    node.is::<Paragraph>()
        && node.children.iter().any(Node::is::<Image>)
        && node.children.iter().all(image_or_space)
}

fn image_or_space(node: &Node) -> bool {
    if node.is::<Image>() {
        let mut has_note = false;
        node.walk(|node, _| has_note |= node.is::<InlineFootnote>());
        !has_note
    } else {
        node.is::<Softbreak>()
            || node
                .cast::<markdown_it::parser::inline::Text>()
                .is_some_and(|text| text.content.trim().is_empty())
    }
}
