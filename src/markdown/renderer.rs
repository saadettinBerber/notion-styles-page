mod anchors;
mod glossary;
mod inline_terms;
mod tree;

use std::collections::HashMap;

use markdown_it::common::utils::escape_html;
use markdown_it::parser::extset::RenderExtSet;
use markdown_it::plugins::cmark::block::fence::CodeFence;
use markdown_it::plugins::cmark::inline::image::Image;
use markdown_it::plugins::cmark::inline::link::Link;
use markdown_it::plugins::html::html_block::HtmlBlock;
use markdown_it::plugins::html::html_inline::HtmlInline;
use markdown_it::{Node, Renderer};
use markdown_it_footnote::definitions::FootnoteDefinition;
use serde::Serialize;

use crate::{MediaSource, PageError, ReaderOptions};

use super::blocks::{Container, SUMMARY_CLOSE, SUMMARY_OPEN, Summary, TOGGLE_BODY_OPEN};
use super::footnotes::{Backlinks, Reference, note_id};
use super::highlight::render_fence;
use super::terms::{self, TermDefinition, Terms};
use anchors::Anchors;
use glossary::Glossary;
use tree::Capture;

pub use tree::RenderNode;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RenderedBody {
    pub html: String,
    pub render_nodes: Vec<RenderNode>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[cfg(test)]
pub(super) fn render(
    root: &Node,
    media: &(impl MediaSource + ?Sized),
) -> Result<String, PageError> {
    Ok(render_with_warnings(root, media)?.html)
}

pub(super) fn render_with_warnings(
    root: &Node,
    media: &(impl MediaSource + ?Sized),
) -> Result<RenderedBody, PageError> {
    let mut renderer = BodyRenderer::new(media, None, Anchors::default(), Glossary::new(root, ""));
    renderer.contents(&root.children);
    renderer.term_cards(root);
    let warnings = renderer.glossary.warnings();
    Ok(RenderedBody {
        html: renderer.finish()?,
        render_nodes: Vec::new(),
        warnings,
    })
}

pub(super) fn render_document(
    root: &Node,
    media: &(impl MediaSource + ?Sized),
    options: &ReaderOptions,
) -> Result<RenderedBody, PageError> {
    let anchors = Anchors::new(root, &options.id_prefix);
    let mut renderer = BodyRenderer::new(
        media,
        Some(options),
        anchors,
        Glossary::new(root, &options.id_prefix),
    );
    let mut render_nodes = renderer.tree_nodes(&root.children);
    let start = renderer.output.len();
    renderer.term_cards(root);
    if renderer.output.len() > start {
        render_nodes.push(RenderNode::Leaf {
            kind: "glossary".into(),
            html: renderer.output[start..].replace('\0', "\u{FFFD}"),
            shared: false,
            inline: false,
        });
    }
    let warnings = renderer.glossary.warnings();
    Ok(RenderedBody {
        html: renderer.finish()?,
        render_nodes,
        warnings,
    })
}

struct BodyRenderer<'media, M: ?Sized> {
    media: &'media M,
    options: Option<&'media ReaderOptions>,
    anchors: Anchors,
    glossary: Glossary,
    output: String,
    error: Option<PageError>,
    image_cache: HashMap<String, Option<String>>,
    capture: Option<Capture>,
    extensions: RenderExtSet,
}

impl<'media, M: MediaSource + ?Sized> BodyRenderer<'media, M> {
    fn new(
        media: &'media M,
        options: Option<&'media ReaderOptions>,
        anchors: Anchors,
        glossary: Glossary,
    ) -> Self {
        Self {
            media,
            options,
            anchors,
            glossary,
            output: String::new(),
            error: None,
            image_cache: HashMap::new(),
            capture: None,
            extensions: RenderExtSet::new(),
        }
    }

    fn finish(self) -> Result<String, PageError> {
        self.error
            .map_or_else(|| Ok(self.output.replace('\0', "\u{FFFD}")), Err)
    }

    fn prefix(&self) -> &'media str {
        self.options
            .map_or("", |options| options.id_prefix.as_str())
    }

    fn element(&mut self, node: &Node) {
        if node.is::<Terms>() {
            return;
        }
        let scope = self.glossary.enter(node);
        if !self.glossary.text_node(node, &mut self.output) {
            self.render_element(node);
        }
        self.glossary.leave(scope);
    }

    fn render_sequence(&mut self, nodes: &[Node]) {
        if self.glossary.has_inline_prose() {
            self.render_inline_terms(nodes);
        } else {
            for node in nodes {
                if self.error.is_some() {
                    break;
                }
                self.element(node);
            }
        }
    }

    fn render_element(&mut self, node: &Node) {
        if let Some(block) = node.cast::<Container>() {
            self.container(block, node);
        } else if node.is::<Summary>() {
            self.text_raw(SUMMARY_OPEN);
            self.contents(&node.children);
            self.text_raw(SUMMARY_CLOSE);
            self.text_raw(TOGGLE_BODY_OPEN);
        } else if let Some(fence) = node.cast::<CodeFence>() {
            render_fence(fence, node, self);
        } else if self.options.is_some() {
            self.reader_element(node);
        } else {
            node.node_value.render(node, self);
        }
    }

    fn term_cards(&mut self, root: &Node) {
        for (index, definition) in terms::definitions(root).iter().enumerate() {
            let term = &definition.cast::<TermDefinition>().unwrap().term;
            self.cr();
            self.text_raw("<div id=\"");
            self.output
                .push_str(&escape_html(&self.glossary.ids[index]));
            self.text_raw("\" class=\"term-card\" popover><strong>");
            self.text(term);
            self.text_raw("</strong>");
            let scope = self.glossary.enter(definition);
            self.contents(&definition.children);
            self.glossary.leave(scope);
            self.text_raw("</div>\n");
        }
    }

    fn container(&mut self, block: &Container, node: &Node) {
        if self.open_container(block) {
            self.contents(&node.children);
            self.text_raw(block.closing());
        }
    }

    fn open_container(&mut self, block: &Container) -> bool {
        match block.opening(self.media) {
            Ok(opening) => {
                self.cr();
                self.text_raw(&opening);
                true
            }
            Err(error) => {
                self.error = Some(error);
                false
            }
        }
    }

    fn reader_element(&mut self, node: &Node) {
        if let Some(image) = node.cast::<Image>() {
            self.image(node, image);
        } else if let Some(link) = node.cast::<Link>() {
            self.link(node, link);
        } else if let Some(reference) = node.cast::<Reference>() {
            self.text_raw(&reference.html(self.prefix()));
        } else if let Some(backlinks) = node.cast::<Backlinks>() {
            for part in backlinks.parts(self.prefix()) {
                self.text_raw(&part);
            }
        } else if let Some(note) = node.cast::<FootnoteDefinition>() {
            self.note_definition(node, note);
        } else if !self.raw_html_allowed() && self.escape_raw_html(node) {
        } else {
            node.node_value.render(node, self);
        }
    }

    fn raw_html_allowed(&self) -> bool {
        self.options.is_none_or(|options| options.allow_raw_html)
    }

    fn escape_raw_html(&mut self, node: &Node) -> bool {
        if let Some(block) = node.cast::<HtmlBlock>() {
            self.cr();
            self.text_raw("<p>");
            self.text(block.content.trim_end_matches(['\r', '\n']));
            self.text_raw("</p>\n");
        } else if let Some(inline) = node.cast::<HtmlInline>() {
            self.text(&inline.content);
        } else {
            return false;
        }
        true
    }

    fn image(&mut self, node: &Node, image: &Image) {
        let result = self
            .image_cache
            .get(&image.url)
            .cloned()
            .map_or_else(|| self.media.embedded_image(&image.url), Ok);
        let embedded = match result {
            Ok(value) => value,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        self.image_cache.insert(image.url.clone(), embedded.clone());
        let Some(url) = embedded else {
            node.node_value.render(node, self);
            return;
        };
        let mut attrs = node.attrs.clone();
        attrs.push(("src", url));
        attrs.push(("alt", node.collect_text()));
        if let Some(title) = &image.title {
            attrs.push(("title", title.clone()));
        }
        attrs.push(("data-source", image.url.clone()));
        self.self_close("img", &attrs);
    }

    fn link(&mut self, node: &Node, link: &Link) {
        let Some(target) = self.anchors.fragment(&link.url) else {
            node.node_value.render(node, self);
            return;
        };
        let mut attrs = node.attrs.clone();
        attrs.push(("href", target));
        if let Some(title) = &link.title {
            attrs.push(("title", title.clone()));
        }
        self.open("a", &attrs);
        self.contents(&node.children);
        self.close("a");
    }

    fn note_definition(&mut self, node: &Node, note: &FootnoteDefinition) {
        let mut attrs = node.attrs.clone();
        if let Some(ordinal) = note.def_id {
            attrs.push(("id", note_id("fn", self.prefix(), &ordinal.to_string())));
        }
        attrs.push(("class", "footnote-item".into()));
        self.cr();
        self.open("li", &attrs);
        self.contents(&node.children);
        self.close("li");
        self.cr();
    }

    fn write_attrs(&mut self, attrs: &[(&str, String)]) {
        for (index, (name, value)) in attrs.iter().enumerate() {
            if matches!(*name, "class" | "style") {
                if attrs[..index].iter().any(|(other, _)| other == name) {
                    continue;
                }
                self.write_attr(name, &merged_attr(attrs, name));
            } else {
                self.write_attr(name, value);
            }
        }
    }

    fn write_attr(&mut self, name: &str, value: &str) {
        self.output.push(' ');
        self.output.push_str(&escape_html(name));
        self.output.push_str("=\"");
        self.output.push_str(&escape_html(value));
        self.output.push('"');
    }
}

fn merged_attr(attrs: &[(&str, String)], name: &str) -> String {
    let separator = if name == "class" { " " } else { ";" };
    attrs
        .iter()
        .filter(|(key, _)| *key == name)
        .map(|(_, value)| value.as_str())
        .collect::<Vec<_>>()
        .join(separator)
}

impl<M: MediaSource + ?Sized> Renderer for BodyRenderer<'_, M> {
    fn open(&mut self, tag: &str, attrs: &[(&str, String)]) {
        self.output.push('<');
        self.output.push_str(tag);
        if !self.prefix().is_empty() && matches!(tag, "h1" | "h2" | "h3") {
            let mut attrs = attrs.to_vec();
            for (_, value) in attrs.iter_mut().filter(|(key, _)| *key == "id") {
                value.insert_str(0, self.prefix());
            }
            self.write_attrs(&attrs);
        } else {
            self.write_attrs(attrs);
        }
        self.output.push('>');
    }

    fn close(&mut self, tag: &str) {
        self.output.push_str("</");
        self.output.push_str(tag);
        self.output.push('>');
    }

    fn self_close(&mut self, tag: &str, attrs: &[(&str, String)]) {
        self.output.push('<');
        self.output.push_str(tag);
        self.write_attrs(attrs);
        self.output.push_str(" />");
    }

    fn contents(&mut self, nodes: &[Node]) {
        if let Some(mut capture) = self.capture.take() {
            capture.opening_end = self.output.len();
            capture.children = if capture.image_paragraph {
                self.image_paragraph(nodes)
            } else {
                self.tree_nodes(nodes)
            };
            capture.children_end = self.output.len();
            self.capture = Some(capture);
            return;
        }
        self.render_sequence(nodes);
    }

    fn cr(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn text(&mut self, text: &str) {
        self.output.push_str(&escape_html(text));
    }

    fn text_raw(&mut self, text: &str) {
        self.output.push_str(text);
    }

    fn ext(&mut self) -> &mut RenderExtSet {
        &mut self.extensions
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[derive(Default)]
    struct TestMedia {
        images: Cell<usize>,
    }

    impl MediaSource for TestMedia {
        fn video_data_uri(&self, _: &str) -> Result<String, PageError> {
            unreachable!("fixture has no video")
        }

        fn image_data_uri(&self, _: &str) -> Result<String, PageError> {
            unreachable!("fixture has no video poster")
        }

        fn embedded_image(&self, path: &str) -> Result<Option<String>, PageError> {
            assert_eq!(path, "a.png");
            self.images.set(self.images.get() + 1);
            Ok(Some("data:image/png;base64,AAAA".into()))
        }
    }

    fn document(source: &str) -> Node {
        let mut parser = markdown_it::MarkdownIt::new();
        markdown_it::plugins::cmark::add(&mut parser);
        markdown_it::plugins::html::add(&mut parser);
        super::super::blocks::add(&mut parser);
        super::super::footnotes::add_reader(&mut parser);
        super::super::inline::add(&mut parser);
        parser.parse(source)
    }

    fn assembled(nodes: &[RenderNode]) -> String {
        nodes
            .iter()
            .map(|node| match node {
                RenderNode::Leaf { html, .. } => html.clone(),
                RenderNode::Branch {
                    opening_html,
                    children,
                    closing_html,
                    ..
                } => {
                    format!("{opening_html}{}{closing_html}", assembled(children))
                }
            })
            .collect()
    }

    #[test]
    fn nested_tree_preserves_shared_blocks_inline_title_and_cached_images() {
        let source = ":::toggle More **words**\n> - Read\n>\n>   ![Alt](a.png)\n>\n>   ---\n:::\n\n![Alt](a.png)\n";
        let root = document(source);
        let media = TestMedia::default();
        let rendered = render_document(&root, &media, &ReaderOptions::default()).unwrap();

        assert_eq!(rendered.html, assembled(&rendered.render_nodes));
        assert_eq!(media.images.get(), 1);
        let [
            RenderNode::Branch { kind, children, .. },
            RenderNode::Leaf { shared: true, .. },
        ] = rendered.render_nodes.as_slice()
        else {
            panic!("{:?}", rendered.render_nodes)
        };
        assert_eq!(kind, "container:toggle");
        let [
            RenderNode::Branch {
                children: summary, ..
            },
            RenderNode::Branch { kind: body, .. },
        ] = children.as_slice()
        else {
            panic!("{children:?}")
        };
        assert!(
            matches!(summary.as_slice(), [RenderNode::Leaf { kind, inline: true, .. }] if kind == "container_title")
        );
        assert_eq!(body, "container_body");
        assert!(rendered.html.contains("data-source=\"a.png\""));
    }

    #[test]
    fn prefixes_heading_links_and_notes_without_mutating_the_prepared_document() {
        let source = "# Başlık\n\n[yol](#başlık) [geri](#fnref1:1) [^a] [^a] ^[Ünlü]\n\n[^a]: Not\n\n[^unused]: Unused\n";
        let root = document(source);
        let media = TestMedia::default();
        let prefixed = render_document(
            &root,
            &media,
            &ReaderOptions {
                id_prefix: "tr-".into(),
                allow_raw_html: false,
            },
        )
        .unwrap();
        let default = render_document(&root, &media, &ReaderOptions::default()).unwrap();

        assert_eq!(prefixed.html, assembled(&prefixed.render_nodes));
        assert!(prefixed.html.contains("id=\"tr-başlık\""));
        assert!(prefixed.html.contains("href=\"#tr-ba%C5%9Fl%C4%B1k\""));
        assert!(prefixed.html.contains("href=\"#fnref-tr--1:1\">geri"));
        assert!(prefixed.html.contains("id=\"fnref-tr--1:1\">[1:1]"));
        assert!(prefixed.html.contains("<p>Ünlü <a href=\"#fnref-tr--2\""));
        assert!(!prefixed.html.contains("Unused"));
        assert!(!default.html.contains("tr-"));
        let RenderNode::Branch { children, .. } = prefixed.render_nodes.last().unwrap() else {
            panic!("{:?}", prefixed.render_nodes)
        };
        assert!(
            matches!(&children[1], RenderNode::Branch { identity, ordinal: 2, .. } if identity == &["inline_footnote", "1"])
        );
    }

    #[test]
    fn raw_html_policy_applies_to_text_and_toggle_title() {
        let root = document("<script>unsafe</script>\n\n:::toggle <b>title</b>\ntext\n:::\n");
        let media = TestMedia::default();
        let escaped = render_document(&root, &media, &ReaderOptions::default()).unwrap();
        let raw = render_document(
            &root,
            &media,
            &ReaderOptions {
                id_prefix: String::new(),
                allow_raw_html: true,
            },
        )
        .unwrap();

        assert_eq!(escaped.html, assembled(&escaped.render_nodes));
        assert!(escaped.html.contains("&lt;script&gt;unsafe&lt;/script&gt;"));
        assert!(
            escaped
                .html
                .contains("<summary>&lt;b&gt;title&lt;/b&gt;</summary>")
        );
        assert!(raw.html.contains("<script>unsafe</script>"));
        assert!(raw.html.contains("<summary><b>title</b></summary>"));
    }
}
