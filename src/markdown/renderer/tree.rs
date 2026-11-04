use markdown_it::Node;
use markdown_it::parser::inline::{Text, TextSpecial};
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
use markdown_it::plugins::extra::tables::Table;
use markdown_it::plugins::html::html_block::HtmlBlock;
use markdown_it_footnote::collect::FootnotesContainerNode;
use markdown_it_footnote::definitions::FootnoteDefinition;
use serde::Serialize;

use super::super::blocks::{
    Container, SUMMARY_CLOSE, SUMMARY_OPEN, TOGGLE_BODY_CLOSE, TOGGLE_BODY_OPEN, TOGGLE_CLOSE,
};
use super::super::footnotes::{Backlinks, Reference, UnusedDefinitions, note_id};
use super::BodyRenderer;
use crate::MediaSource;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum RenderNode {
    Leaf {
        kind: String,
        html: String,
        shared: bool,
        inline: bool,
    },
    Branch {
        kind: String,
        opening_html: String,
        children: Vec<Self>,
        closing_html: String,
        anchor_id: String,
        identity: Vec<String>,
        ordinal: usize,
    },
}

fn leaf(kind: &str, html: String, shared: bool, inline: bool) -> RenderNode {
    RenderNode::Leaf {
        kind: kind.into(),
        html,
        shared,
        inline,
    }
}

fn branch(
    kind: String,
    opening_html: String,
    children: Vec<RenderNode>,
    closing_html: String,
) -> RenderNode {
    RenderNode::Branch {
        kind,
        opening_html,
        children,
        closing_html,
        anchor_id: String::new(),
        identity: Vec::new(),
        ordinal: 0,
    }
}

pub(super) struct Capture {
    pub(super) image_paragraph: bool,
    pub(super) opening_end: usize,
    pub(super) children_end: usize,
    pub(super) children: Vec<RenderNode>,
}

impl Capture {
    fn new(start: usize, image_paragraph: bool) -> Self {
        Self {
            image_paragraph,
            opening_end: start,
            children_end: start,
            children: Vec::new(),
        }
    }
}

impl<M: MediaSource + ?Sized> BodyRenderer<'_, M> {
    pub(super) fn tree_nodes(&mut self, mut nodes: &[Node]) -> Vec<RenderNode> {
        let mut tree = Vec::new();
        while let [first, rest @ ..] = nodes {
            if self.error.is_some() {
                break;
            }
            match role(first) {
                Role::Hidden => nodes = rest,
                Role::Inline => {
                    let length = nodes
                        .iter()
                        .take_while(|node| matches!(role(node), Role::Inline))
                        .count();
                    tree.push(self.render_leaf(
                        "paragraph",
                        &nodes[..length],
                        image_only(&nodes[..length]),
                        false,
                    ));
                    nodes = &nodes[length..];
                }
                Role::Backlinks(backlinks) => {
                    tree.extend(self.render_backlinks(backlinks));
                    nodes = rest;
                }
                Role::Container(container) if container.is_toggle() => {
                    tree.push(self.toggle_tree(first, container));
                    nodes = rest;
                }
                other => {
                    tree.push(self.tree_node(first, other));
                    nodes = rest;
                }
            }
        }
        tree
    }

    pub(super) fn image_paragraph(&mut self, children: &[Node]) -> Vec<RenderNode> {
        let count = children
            .iter()
            .take_while(|node| !node.is::<Backlinks>())
            .count();
        let mut nodes = vec![self.render_leaf("image_content", &children[..count], true, true)];
        nodes.extend(self.tree_nodes(&children[count..]));
        nodes
    }

    fn tree_node(&mut self, node: &Node, role: Role<'_>) -> RenderNode {
        match role {
            Role::Leaf { kind, shared } => {
                self.render_leaf(kind, std::slice::from_ref(node), shared, false)
            }
            Role::Container(container) => {
                self.record_branch(node, format!("container:{}", container.name()), false)
            }
            Role::Branch(kind) => self.record_branch(node, kind.into(), false),
            Role::ImageParagraph => self.record_branch(node, "paragraph".into(), true),
            Role::Footnote(note) => self.footnote_tree(node, note),
            _ => unreachable!("tree traversal handles grouped and hidden roles"),
        }
    }

    fn record_branch(&mut self, node: &Node, kind: String, image_paragraph: bool) -> RenderNode {
        let start = self.output.len();
        self.capture = Some(Capture::new(start, image_paragraph));
        self.element(node);
        let record = self
            .capture
            .take()
            .expect("a render branch restores its capture");
        branch(
            kind,
            self.fragment(start, record.opening_end),
            record.children,
            self.fragment(record.children_end, self.output.len()),
        )
    }

    fn footnote_tree(&mut self, node: &Node, note: &FootnoteDefinition) -> RenderNode {
        let mut output = self.record_branch(node, "footnote".into(), false);
        if let RenderNode::Branch {
            anchor_id,
            identity,
            ordinal,
            ..
        } = &mut output
        {
            *ordinal = note.def_id.unwrap_or_default();
            *anchor_id = note_id("fn", self.prefix(), &ordinal.to_string());
            *identity = note.label.as_ref().map_or_else(
                || self.anchors.inline_identity(*ordinal),
                |label| vec!["footnote".into(), label.clone()],
            );
        }
        output
    }

    fn toggle_tree(&mut self, node: &Node, container: &Container) -> RenderNode {
        let start = self.output.len();
        if !self.open_container(container) {
            return leaf("container_error", String::new(), false, false);
        }
        let opening = self.fragment(start, self.output.len());
        let [summary, body @ ..] = node.children.as_slice() else {
            unreachable!("toggle containers always have a summary")
        };
        let summary = self.summary_tree(summary);
        let body = self.toggle_body_tree(body);
        self.output.push_str(TOGGLE_CLOSE);
        branch(
            format!("container:{}", container.name()),
            opening,
            vec![summary, body],
            TOGGLE_CLOSE.into(),
        )
    }

    fn summary_tree(&mut self, node: &Node) -> RenderNode {
        self.output.push_str(SUMMARY_OPEN);
        let title = self.render_leaf("container_title", &node.children, false, true);
        self.output.push_str(SUMMARY_CLOSE);
        branch(
            "summary".into(),
            SUMMARY_OPEN.into(),
            vec![title],
            SUMMARY_CLOSE.into(),
        )
    }

    fn toggle_body_tree(&mut self, body: &[Node]) -> RenderNode {
        self.output.push_str(TOGGLE_BODY_OPEN);
        let children = self.tree_nodes(body);
        self.output.push_str(TOGGLE_BODY_CLOSE);
        branch(
            "container_body".into(),
            TOGGLE_BODY_OPEN.into(),
            children,
            TOGGLE_BODY_CLOSE.into(),
        )
    }

    fn render_leaf(
        &mut self,
        kind: &str,
        nodes: &[Node],
        shared: bool,
        inline: bool,
    ) -> RenderNode {
        let start = self.output.len();
        for node in nodes {
            self.element(node);
            if self.error.is_some() {
                break;
            }
        }
        leaf(
            kind,
            self.fragment(start, self.output.len()),
            shared,
            inline,
        )
    }

    fn render_backlinks(&mut self, backlinks: &Backlinks) -> Vec<RenderNode> {
        let parts: Vec<_> = backlinks.parts(self.prefix()).collect();
        for part in &parts {
            self.output.push_str(part);
        }
        parts
            .into_iter()
            .map(|html| leaf("footnote_anchor", html, false, true))
            .collect()
    }

    fn fragment(&self, start: usize, end: usize) -> String {
        self.output[start..end].replace('\0', "\u{FFFD}")
    }
}

enum Role<'node> {
    Leaf { kind: &'static str, shared: bool },
    Branch(&'static str),
    Container(&'node Container),
    Footnote(&'node FootnoteDefinition),
    Backlinks(&'node Backlinks),
    ImageParagraph,
    Inline,
    Hidden,
}

fn role(node: &Node) -> Role<'_> {
    if let Some(container) = node.cast::<Container>() {
        Role::Container(container)
    } else if let Some(note) = node.cast::<FootnoteDefinition>() {
        Role::Footnote(note)
    } else if let Some(anchors) = node.cast::<Backlinks>() {
        Role::Backlinks(anchors)
    } else if node.is::<UnusedDefinitions>() {
        Role::Hidden
    } else if let Some(kind) = standard_branch(node) {
        Role::Branch(kind)
    } else if node.is::<Paragraph>() && image_paragraph_with_backlinks(&node.children) {
        Role::ImageParagraph
    } else if let Some((kind, shared)) = standard_leaf(node) {
        Role::Leaf { kind, shared }
    } else {
        Role::Inline
    }
}

fn standard_branch(node: &Node) -> Option<&'static str> {
    Some(if node.is::<Blockquote>() {
        "blockquote"
    } else if node.is::<BulletList>() {
        "bullet_list"
    } else if node.is::<OrderedList>() {
        "ordered_list"
    } else if node.is::<ListItem>() {
        "list_item"
    } else if node.is::<FootnotesContainerNode>() {
        "footnote_block"
    } else {
        return None;
    })
}

fn standard_leaf(node: &Node) -> Option<(&'static str, bool)> {
    Some(if node.is::<Paragraph>() {
        ("paragraph", image_only(&node.children))
    } else if node.is::<ATXHeading>() || node.is::<SetextHeader>() {
        ("heading", false)
    } else if node.is::<CodeFence>() {
        ("fence", true)
    } else if node.is::<CodeBlock>() {
        ("code_block", true)
    } else if node.is::<ThematicBreak>() {
        ("hr", true)
    } else if node.is::<Table>() {
        ("table", false)
    } else if node.is::<HtmlBlock>() {
        ("html_block", false)
    } else if node.is::<Definition>() {
        ("definition", false)
    } else {
        return None;
    })
}

fn image_paragraph_with_backlinks(nodes: &[Node]) -> bool {
    let first = nodes.iter().position(|node| node.is::<Backlinks>());
    first.is_some_and(|split| {
        nodes[split..].iter().all(|node| node.is::<Backlinks>()) && image_only(&nodes[..split])
    })
}

fn image_only(nodes: &[Node]) -> bool {
    nodes.iter().any(Node::is::<Image>) && nodes.iter().all(image_or_space)
}

fn image_or_space(node: &Node) -> bool {
    if node.is::<Image>() {
        let mut has_footnotes = false;
        node.walk(|child, _| has_footnotes |= child.is::<Reference>());
        !has_footnotes
    } else if let Some(text) = node.cast::<Text>() {
        text.content.trim().is_empty()
    } else if let Some(text) = node.cast::<TextSpecial>() {
        text.content.trim().is_empty()
    } else {
        node.is::<Softbreak>()
    }
}
