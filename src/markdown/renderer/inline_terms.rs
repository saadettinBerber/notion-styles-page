//! Match visible inline runs before rendering, keeping formatting inside valid buttons.

use markdown_it::Node;
use markdown_it::Renderer;
use markdown_it::parser::inline::{Text, TextSpecial};
use markdown_it::plugins::cmark::inline::emphasis::{Em, Strong};
use markdown_it::plugins::cmark::inline::newline::Softbreak;
use markdown_it::plugins::extra::strikethrough::Strikethrough;

use super::super::inline::Mark;
use super::BodyRenderer;
use crate::MediaSource;

#[derive(Clone, Copy)]
struct Wrapper<'a> {
    node: &'a Node,
    tag: &'static str,
}

enum Content<'a> {
    Text(&'a str),
    Softbreak,
    Boundary(&'a Node),
}

struct Atom<'a> {
    content: Content<'a>,
    wrappers: Vec<Wrapper<'a>>,
}

fn atoms<'a>(nodes: &'a [Node], wrappers: &mut Vec<Wrapper<'a>>, output: &mut Vec<Atom<'a>>) {
    for node in nodes {
        let content = if let Some(text) = node.cast::<Text>() {
            Content::Text(&text.content)
        } else if let Some(text) = node.cast::<TextSpecial>() {
            Content::Text(&text.content)
        } else if node.is::<Softbreak>() {
            Content::Softbreak
        } else if let Some(tag) = formatting(node) {
            wrappers.push(Wrapper { node, tag });
            atoms(&node.children, wrappers, output);
            wrappers.pop();
            continue;
        } else {
            Content::Boundary(node)
        };
        output.push(Atom {
            content,
            wrappers: wrappers.clone(),
        });
    }
}

fn formatting(node: &Node) -> Option<&'static str> {
    if node.is::<Em>() {
        Some("em")
    } else if node.is::<Strong>() {
        Some("strong")
    } else if node.is::<Strikethrough>() {
        Some("s")
    } else if node.is::<Mark>() {
        Some("mark")
    } else {
        None
    }
}

impl<M: MediaSource + ?Sized> BodyRenderer<'_, M> {
    pub(super) fn render_inline_terms(&mut self, nodes: &[Node]) {
        let mut flattened = Vec::new();
        atoms(nodes, &mut Vec::new(), &mut flattened);
        let mut open = Vec::new();
        let mut start = 0;
        for (index, atom) in flattened.iter().enumerate() {
            if let Content::Boundary(node) = atom.content {
                self.inline_run(&flattened[start..index], &mut open);
                self.sync_wrappers(&mut open, &atom.wrappers);
                self.element(node);
                start = index + 1;
                if self.error.is_some() {
                    break;
                }
            }
        }
        if self.error.is_none() {
            self.inline_run(&flattened[start..], &mut open);
        }
        self.sync_wrappers(&mut open, &[]);
    }

    fn inline_run<'a>(&mut self, atoms: &[Atom<'a>], open: &mut Vec<Wrapper<'a>>) {
        let mut text = String::new();
        let mut offsets = Vec::with_capacity(atoms.len());
        for atom in atoms {
            offsets.push(text.len());
            match atom.content {
                Content::Text(content) => text.push_str(content),
                Content::Softbreak => text.push(' '),
                Content::Boundary(_) => unreachable!("runs stop at protected inline nodes"),
            }
        }
        let matches = self.glossary.match_run(&text);
        let mut cursor = 0;
        for found in matches {
            self.inline_range(atoms, &offsets, cursor, found.start, open);
            self.sync_wrappers(open, &[]);
            self.open(
                "button",
                &[
                    ("type", "button".into()),
                    ("class", "term".into()),
                    ("popovertarget", self.glossary.ids[found.term_index].clone()),
                ],
            );
            self.inline_range(atoms, &offsets, found.start, found.end, open);
            self.sync_wrappers(open, &[]);
            self.close("button");
            cursor = found.end;
        }
        self.inline_range(atoms, &offsets, cursor, text.len(), open);
    }

    fn inline_range<'a>(
        &mut self,
        atoms: &[Atom<'a>],
        offsets: &[usize],
        start: usize,
        end: usize,
        open: &mut Vec<Wrapper<'a>>,
    ) {
        if start == end {
            return;
        }
        for (atom, &offset) in atoms.iter().zip(offsets) {
            let length = match atom.content {
                Content::Text(content) => content.len(),
                Content::Softbreak => 1,
                Content::Boundary(_) => unreachable!("runs stop at protected inline nodes"),
            };
            let segment_start = start.max(offset);
            let segment_end = end.min(offset + length);
            if segment_start >= segment_end {
                continue;
            }
            self.sync_wrappers(open, &atom.wrappers);
            match atom.content {
                Content::Text(content) => {
                    self.text(&content[segment_start - offset..segment_end - offset]);
                }
                Content::Softbreak => self.cr(),
                Content::Boundary(_) => unreachable!("runs stop at protected inline nodes"),
            }
        }
    }

    fn sync_wrappers<'a>(&mut self, open: &mut Vec<Wrapper<'a>>, desired: &[Wrapper<'a>]) {
        let common = open
            .iter()
            .zip(desired)
            .take_while(|(left, right)| std::ptr::eq(left.node, right.node))
            .count();
        for wrapper in open.drain(common..).rev() {
            self.close(wrapper.tag);
        }
        for wrapper in &desired[common..] {
            self.open(wrapper.tag, &wrapper.node.attrs);
            open.push(*wrapper);
        }
    }
}
