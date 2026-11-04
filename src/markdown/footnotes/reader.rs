use std::collections::HashSet;

use markdown_it::parser::core::CoreRule;
use markdown_it::parser::inline::{InlineRule, InlineState};
use markdown_it::plugins::cmark::block::paragraph::Paragraph;
use markdown_it::{MarkdownIt, Node, NodeValue};
use markdown_it_footnote::FootnoteMap;
use markdown_it_footnote::collect::FootnotesContainerNode;
use markdown_it_footnote::definitions::FootnoteDefinition;
use markdown_it_footnote::inline::InlineFootnote;
use markdown_it_footnote::references::FootnoteReference;

pub(super) fn add(parser: &mut MarkdownIt) {
    markdown_it_footnote::definitions::add(parser);
    markdown_it_footnote::references::add(parser);
    parser.inline.add_rule::<InlineNoteRule>();
    parser.add_rule::<CollectReaderFootnotes>();
    markdown_it_footnote::back_refs::add(parser);
}

#[derive(Debug)]
pub(in crate::markdown) struct UnusedDefinitions;
impl NodeValue for UnusedDefinitions {}

pub(in crate::markdown) fn unused_definitions(root: &Node) -> &[Node] {
    root.children
        .iter()
        .find(|child| child.is::<UnusedDefinitions>())
        .map_or(&[], |node| &node.children)
}

struct CollectReaderFootnotes;
impl CoreRule for CollectReaderFootnotes {
    fn run(root: &mut Node, _: &MarkdownIt) {
        let mut referenced = HashSet::new();
        root.walk(|node, _| {
            if let Some(reference) = node.cast::<FootnoteReference>() {
                referenced.insert(reference.def_id);
            }
        });
        let mut rendered = Node::new(FootnotesContainerNode);
        let mut unused = Node::new(UnusedDefinitions);
        collect_definitions(
            root,
            &referenced,
            &mut rendered.children,
            &mut unused.children,
        );
        if !rendered.children.is_empty() {
            root.children.push(rendered);
        }
        if !unused.children.is_empty() {
            root.children.push(unused);
        }
    }
}

fn collect_definitions(
    node: &mut Node,
    referenced: &HashSet<usize>,
    rendered: &mut Vec<Node>,
    unused: &mut Vec<Node>,
) {
    let children = std::mem::take(&mut node.children);
    for mut child in children {
        collect_definitions(&mut child, referenced, rendered, unused);
        let Some(note) = child.cast::<FootnoteDefinition>() else {
            node.children.push(child);
            continue;
        };
        if !note.def_id.is_some_and(|id| referenced.contains(&id)) {
            unused.push(child);
        } else {
            if note.inline {
                let mut paragraph = Node::new(Paragraph);
                paragraph.children = std::mem::take(&mut child.children);
                child.children.push(paragraph);
            }
            rendered.push(child);
        }
    }
}

struct InlineNoteRule;
impl InlineRule for InlineNoteRule {
    const MARKER: char = '^';

    fn check(state: &mut InlineState) -> Option<usize> {
        state.src[state.pos..state.pos_max].strip_prefix("^[")?;
        let start = state.pos;
        state.pos += 2;
        state.level += 1;
        let end = find_closing_bracket(state);
        state.level -= 1;
        state.pos = start;
        end.map(|position| position + 1 - start)
    }

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let len = Self::check(state)?;
        let (def_id, ref_id) = state
            .root_ext
            .get_or_insert_default::<FootnoteMap>()
            .add_inline_def();
        let start = state.pos + 2;
        let end = state.pos + len - 1;
        let definition = inline_definition(state, start, end, def_id);
        let reference = Node::new(FootnoteReference {
            label: None,
            def_id,
            ref_id,
        });
        let mut note = Node::new(InlineFootnote);
        note.children = vec![definition, reference];
        Some((note, len))
    }
}

fn find_closing_bracket(state: &mut InlineState) -> Option<usize> {
    let mut depth = 0;
    while state.pos < state.pos_max {
        let start = state.pos;
        let marker = state.src.as_bytes()[start];
        if marker == b']' {
            if depth == 0 {
                return Some(start);
            }
            depth -= 1;
        }
        state.md.inline.skip_token(state);
        if marker == b'[' && state.pos == start + 1 {
            depth += 1;
        }
    }
    None
}

fn inline_definition(state: &mut InlineState, start: usize, end: usize, def_id: usize) -> Node {
    let mut node = Node::new(FootnoteDefinition {
        label: None,
        def_id: Some(def_id),
        inline: true,
    });
    node.srcmap = state.get_map(start, end);
    let parent = std::mem::replace(&mut state.node, node);
    let outer_start = std::mem::replace(&mut state.pos, start);
    let outer_end = std::mem::replace(&mut state.pos_max, end);
    state.level += 1;
    state.md.inline.tokenize(state);
    state.level -= 1;
    state.pos = outer_start;
    state.pos_max = outer_end;
    std::mem::replace(&mut state.node, parent)
}
