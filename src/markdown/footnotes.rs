mod reader;

use std::collections::HashMap;

use markdown_it::common::utils::escape_html;
use markdown_it::parser::core::CoreRule;
use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};
use markdown_it_footnote::back_refs::FootnoteRefAnchor;
use markdown_it_footnote::collect::FootnotesContainerNode;
use markdown_it_footnote::definitions::FootnoteDefinition;
use markdown_it_footnote::references::FootnoteReference;

pub(super) use reader::{UnusedDefinitions, unused_definitions};

pub(super) fn add(parser: &mut MarkdownIt) {
    markdown_it_footnote::add(parser);
    parser.add_rule::<NumberByFirstUse>();
}

pub(super) fn add_reader(parser: &mut MarkdownIt) {
    reader::add(parser);
    parser.add_rule::<NumberByFirstUse>();
}

struct NumberByFirstUse;

impl CoreRule for NumberByFirstUse {
    fn run(root: &mut Node, _: &MarkdownIt) {
        let mut notes = HashMap::new();
        let mut references = HashMap::new();
        root.walk_mut(|node, _| {
            if let Some(original) = node.cast::<FootnoteReference>() {
                let next_ordinal = notes.len() + 1;
                let (ordinal, repeats) = notes.entry(original.def_id).or_insert((next_ordinal, 0));
                let reference = Reference {
                    ordinal: *ordinal,
                    repeat: *repeats,
                };
                *repeats += 1;
                references.insert(original.ref_id, reference);
                node.replace(reference);
            }
        });
        update_definitions(root, &notes, &references);
    }
}

fn update_definitions(
    root: &mut Node,
    notes: &HashMap<usize, (usize, usize)>,
    references: &HashMap<usize, Reference>,
) {
    root.walk_mut(|node, _| {
        if let Some(note) = node.cast_mut::<FootnoteDefinition>() {
            note.def_id = note
                .def_id
                .and_then(|id| notes.get(&id).map(|&(ordinal, _)| ordinal));
        } else if let Some(anchors) = node.cast::<FootnoteRefAnchor>() {
            let refs = anchors
                .ref_ids
                .iter()
                .filter_map(|id| references.get(id).copied());
            node.replace(Backlinks(refs.collect()));
        }
    });
    if let Some(list) = root
        .children
        .iter_mut()
        .find(|node| node.is::<FootnotesContainerNode>())
    {
        list.children.sort_by_key(|node| {
            node.cast::<FootnoteDefinition>()
                .and_then(|note| note.def_id)
        });
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Reference {
    ordinal: usize,
    repeat: usize,
}

impl Reference {
    pub(super) fn label(self) -> String {
        if self.repeat == 0 {
            return self.ordinal.to_string();
        }
        format!("{}:{}", self.ordinal, self.repeat)
    }

    pub(super) fn target(self, prefix: &str) -> String {
        note_id("fn", prefix, &self.ordinal.to_string())
    }

    pub(super) fn id(self, prefix: &str) -> String {
        note_id("fnref", prefix, &self.label())
    }

    pub(super) fn html(self, prefix: &str) -> String {
        format!(
            "<sup class=\"footnote-ref\"><a href=\"#{}\" id=\"{}\">[{}]</a></sup>",
            escape_html(&self.target(prefix)),
            escape_html(&self.id(prefix)),
            self.label()
        )
    }
}

impl NodeValue for Reference {
    fn render(&self, _: &Node, html: &mut dyn Renderer) {
        html.text_raw(&self.html(""));
    }
}

#[derive(Debug)]
pub(super) struct Backlinks(Vec<Reference>);

impl Backlinks {
    pub(super) fn parts<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = String> + 'a {
        self.0.iter().map(move |reference| {
            format!(
                " <a href=\"#{}\" class=\"footnote-backref\">↩\u{fe0e}</a>",
                escape_html(&reference.id(prefix))
            )
        })
    }
}

impl NodeValue for Backlinks {
    fn render(&self, _: &Node, html: &mut dyn Renderer) {
        for part in self.parts("") {
            html.text_raw(&part);
        }
    }
}

pub(super) fn note_id(kind: &str, prefix: &str, number: &str) -> String {
    if prefix.is_empty() {
        return format!("{kind}{number}");
    }
    format!("{kind}-{prefix}-{number}")
}
