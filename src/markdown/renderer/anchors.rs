use std::collections::HashMap;

use markdown_it::Node;
use markdown_it::plugins::cmark::block::heading::ATXHeading;
use markdown_it::plugins::cmark::block::lheading::SetextHeader;
use markdown_it_footnote::definitions::FootnoteDefinition;
use percent_encoding::percent_decode_str;

use super::super::footnotes::{Reference, UnusedDefinitions, note_id};

#[derive(Default)]
pub(super) struct Anchors {
    targets: HashMap<String, String>,
    inline_order: HashMap<usize, usize>,
}

impl Anchors {
    pub(super) fn new(root: &Node, prefix: &str) -> Self {
        let mut anchors = Self::default();
        let mut headings = Vec::new();
        let mut inline = Vec::new();
        collect(
            root,
            prefix,
            &mut anchors.targets,
            &mut headings,
            &mut inline,
        );
        inline.sort_unstable();
        for (index, (_, ordinal)) in inline.into_iter().enumerate() {
            anchors.inline_order.insert(ordinal, index + 1);
        }
        anchors.targets.extend(headings);
        anchors
    }

    pub(super) fn fragment(&self, href: &str) -> Option<String> {
        let id = percent_decode_str(href.strip_prefix('#')?)
            .decode_utf8()
            .ok()?;
        let target = self.targets.get(id.as_ref())?;
        Some(encoded_fragment(target))
    }

    pub(super) fn inline_identity(&self, ordinal: usize) -> Vec<String> {
        self.inline_order
            .get(&ordinal)
            .map_or_else(Vec::new, |index| {
                vec!["inline_footnote".into(), index.to_string()]
            })
    }
}

fn collect(
    node: &Node,
    prefix: &str,
    targets: &mut HashMap<String, String>,
    headings: &mut Vec<(String, String)>,
    inline: &mut Vec<(usize, usize)>,
) {
    if node.is::<UnusedDefinitions>() {
        return;
    }
    if let Some(reference) = node.cast::<Reference>() {
        if !prefix.is_empty() {
            targets.insert(reference.id(""), reference.id(prefix));
        }
    } else if let Some(note) = node.cast::<FootnoteDefinition>() {
        if let Some(ordinal) = note.def_id {
            add_note(node, note, ordinal, prefix, targets, inline);
        }
    } else if !prefix.is_empty() && (node.is::<ATXHeading>() || node.is::<SetextHeader>()) {
        for (_, value) in node.attrs.iter().filter(|(key, _)| *key == "id") {
            headings.push((value.clone(), format!("{prefix}{value}")));
        }
    }
    for child in &node.children {
        collect(child, prefix, targets, headings, inline);
    }
}

fn add_note(
    node: &Node,
    note: &FootnoteDefinition,
    ordinal: usize,
    prefix: &str,
    targets: &mut HashMap<String, String>,
    inline: &mut Vec<(usize, usize)>,
) {
    if !prefix.is_empty() {
        let ordinal = ordinal.to_string();
        targets.insert(note_id("fn", "", &ordinal), note_id("fn", prefix, &ordinal));
    }
    if note.inline {
        let start = node
            .srcmap
            .map_or(usize::MAX, |span| span.get_byte_offsets().0);
        inline.push((start, ordinal));
    }
}

fn encoded_fragment(value: &str) -> String {
    use std::fmt::Write as _;

    let mut href = String::from("#");
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~:".contains(&byte) {
            href.push(char::from(byte));
        } else {
            write!(href, "%{byte:02X}").expect("writing into a string cannot fail");
        }
    }
    href
}
