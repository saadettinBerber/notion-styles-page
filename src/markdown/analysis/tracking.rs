use std::ops::Range;

use markdown_it::common::utils::calc_right_whitespace_with_tabstops;
use markdown_it::generics::inline::full_link::{
    LinkPrefixScanner, LinkScanner, parse_link_destination,
};
use markdown_it::parser::block::{BlockRule, BlockState};
use markdown_it::parser::extset::NodeExt;
use markdown_it::parser::inline::{InlineRule, InlineState};
use markdown_it::plugins::cmark::block::code::CodeScanner;
use markdown_it::plugins::cmark::block::fence::{CodeFence, FenceScanner};
use markdown_it::plugins::cmark::block::reference::{Definition, ReferenceScanner};
use markdown_it::{MarkdownIt, Node};

use super::source::line_ending;

#[derive(Debug)]
pub(super) struct DestinationSpan(pub Range<usize>);
impl NodeExt for DestinationSpan {}

#[derive(Debug)]
pub(super) struct ImageLabel(pub String);
impl NodeExt for ImageLabel {}

#[derive(Debug)]
pub(super) struct CodeBytes(pub Vec<u8>);
impl NodeExt for CodeBytes {}

pub(super) fn configure(parser: &mut MarkdownIt) {
    parser
        .block
        .add_rule::<TrackedReference>()
        .before::<ReferenceScanner>();
    parser
        .block
        .add_rule::<TrackedCode>()
        .before::<CodeScanner>();
    parser
        .block
        .add_rule::<TrackedFence>()
        .before::<FenceScanner>();
    parser
        .inline
        .add_rule::<TrackedLink<false>>()
        .before::<LinkScanner<false>>();
    parser
        .inline
        .add_rule::<TrackedLink<true>>()
        .before::<LinkPrefixScanner<'!', true>>();
}

struct TrackedReference;

impl BlockRule for TrackedReference {
    fn check(state: &mut BlockState) -> Option<()> {
        ReferenceScanner::check(state)
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let (mut node, count) = ReferenceScanner::run(state)?;
        let definition = node.cast::<Definition>()?;
        let (text, mapping) =
            state.get_lines(state.line, state.line + count, state.blk_indent, false);
        let opening = text.find('[')?;
        let label_end = opening + definition.label.len() + 3;
        if let Some(span) = destination(&text, label_end) {
            let start = mapped_offset(&mapping, span.start);
            let end = mapped_offset(&mapping, span.end);
            node.ext.insert(DestinationSpan(start..end));
        }
        Some((node, count))
    }
}

fn mapped_offset(mapping: &[(usize, usize)], position: usize) -> usize {
    let (line, original) = mapping[mapping.partition_point(|(line, _)| *line <= position) - 1];
    original + position - line
}

struct TrackedCode;

impl BlockRule for TrackedCode {
    fn check(state: &mut BlockState) -> Option<()> {
        CodeScanner::check(state)
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let (mut node, count) = CodeScanner::run(state)?;
        let lines = state.line..state.line + count;
        node.ext
            .insert(CodeBytes(code_bytes(state, lines, state.blk_indent + 4)));
        Some((node, count))
    }
}

struct TrackedFence;

impl BlockRule for TrackedFence {
    fn check(state: &mut BlockState) -> Option<()> {
        FenceScanner::check(state)
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let (mut node, count) = FenceScanner::run(state)?;
        let fence = node.cast::<CodeFence>()?;
        let start = state.line + 1;
        let end = start + fence.content.bytes().filter(|&byte| byte == b'\n').count();
        let indent = state.line_offsets[state.line].indent_nonspace as usize;
        node.ext
            .insert(CodeBytes(code_bytes(state, start..end, indent)));
        Some((node, count))
    }
}

fn code_bytes(state: &BlockState, lines: Range<usize>, indent: usize) -> Vec<u8> {
    lines
        .flat_map(|line| source_code_line(state, line, indent).bytes())
        .collect()
}

fn source_code_line<'a>(state: &'a BlockState, line: usize, indent: usize) -> &'a str {
    let offset = &state.line_offsets[line];
    let prefix = &state.src[offset.line_start..offset.first_nonspace];
    let (partial_tab, first) =
        calc_right_whitespace_with_tabstops(prefix, offset.indent_nonspace - indent as i32);
    let start = offset.line_start + first;
    let start = start.saturating_sub(usize::from(
        partial_tab > 0 && prefix.as_bytes().get(first.saturating_sub(1)) == Some(&b'\t'),
    ));
    let end = line_ending(state.src, offset.line_end);
    &state.src[start..end]
}

struct TrackedLink<const IMAGE: bool>;

impl<const IMAGE: bool> InlineRule for TrackedLink<IMAGE> {
    const MARKER: char = if IMAGE { '!' } else { '[' };

    fn check(state: &mut InlineState) -> Option<usize> {
        if IMAGE {
            LinkPrefixScanner::<'!', true>::check(state)
        } else {
            LinkScanner::<false>::check(state)
        }
    }

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let (mut node, length) = if IMAGE {
            LinkPrefixScanner::<'!', true>::run(state)?
        } else {
            LinkScanner::<false>::run(state)?
        };
        let start = state.pos + if IMAGE { 2 } else { 1 };
        if let Some(label_end) = label_end(state, start) {
            record_link(&mut node, state, start..label_end, IMAGE);
        }
        Some((node, length))
    }
}

fn record_link(node: &mut Node, state: &InlineState, label: Range<usize>, image: bool) {
    if image {
        node.ext
            .insert(ImageLabel(state.src[label.clone()].to_owned()));
    }
    if state.src.as_bytes().get(label.end + 1) == Some(&b'(')
        && let Some(span) = destination(&state.src, label.end + 2)
        && let Some(span) = state.get_map(span.start, span.end)
    {
        let (start, end) = span.get_byte_offsets();
        node.ext.insert(DestinationSpan(start..end));
    }
}

fn label_end(state: &mut InlineState, start: usize) -> Option<usize> {
    let saved = std::mem::replace(&mut state.pos, start);
    let result = scan_label(state);
    state.pos = saved;
    result
}

fn scan_label(state: &mut InlineState) -> Option<usize> {
    let mut depth = 1;
    while state.pos < state.pos_max {
        let byte = state.src.as_bytes()[state.pos];
        if byte == b']' {
            depth -= 1;
            if depth == 0 {
                return Some(state.pos);
            }
        }
        let start = state.pos;
        state.md.inline.skip_token(state);
        if byte == b'[' && state.pos == start + 1 {
            depth += 1;
        }
    }
    None
}

fn destination(source: &str, start: usize) -> Option<Range<usize>> {
    let raw = source.get(start..)?;
    let offset = raw.trim_start_matches([' ', '\t', '\n']).len();
    let start = source.len() - offset;
    let parsed = parse_link_destination(source, start, source.len())?;
    let span = if source.as_bytes().get(start) == Some(&b'<') {
        start + 1..parsed.pos - 1
    } else {
        start..parsed.pos
    };
    (!span.is_empty()).then_some(span)
}
