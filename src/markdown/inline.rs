use std::collections::HashSet;

use markdown_it::parser::core::CoreRule;
use markdown_it::parser::inline::{InlineRule, InlineState, Text, TextSpecial};
use markdown_it::plugins::cmark::block::heading::ATXHeading;
use markdown_it::plugins::cmark::block::lheading::SetextHeader;
use markdown_it::plugins::cmark::block::list::{BulletList, ListItem, OrderedList};
use markdown_it::plugins::cmark::block::paragraph::Paragraph;
use markdown_it::plugins::cmark::inline::image::Image;
use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};

pub(super) fn add(parser: &mut MarkdownIt) {
    parser.inline.add_rule::<MarkRule>();
    parser.add_rule::<HeadingAnchors>();
    parser.add_rule::<TaskLists>();
}

#[derive(Debug)]
struct Mark;
impl NodeValue for Mark {
    fn render(&self, node: &Node, html: &mut dyn Renderer) {
        html.open("mark", &node.attrs);
        html.contents(&node.children);
        html.close("mark");
    }
}

struct MarkRule;
impl InlineRule for MarkRule {
    const MARKER: char = '=';

    fn check(state: &mut InlineState) -> Option<usize> {
        let after_marker = state.src[state.pos..state.pos_max].strip_prefix("==")?;
        let closing = after_marker.find("==")?;
        (closing != 0).then_some(closing + 4)
    }

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        let len = Self::check(state)?;
        let start = state.pos;
        let end = std::mem::replace(&mut state.pos_max, start + len - 2);
        let parent = std::mem::replace(&mut state.node, Node::new(Mark));

        state.pos += 2;
        state.level += 1;
        state.md.inline.tokenize(state);
        state.level -= 1;
        state.pos = start;
        state.pos_max = end;

        Some((std::mem::replace(&mut state.node, parent), len))
    }
}

struct HeadingAnchors;
impl CoreRule for HeadingAnchors {
    fn run(root: &mut Node, _: &MarkdownIt) {
        let mut taken = HashSet::new();
        root.walk_mut(|node, _| {
            if heading_level(node).is_some_and(|level| level <= 3) {
                let slug = heading_slug(node);
                node.attrs.push(("id", unique_slug(slug, &mut taken)));
            }
        });
    }
}

fn heading_level(node: &Node) -> Option<u8> {
    node.cast::<ATXHeading>()
        .map(|heading| heading.level)
        .or_else(|| node.cast::<SetextHeader>().map(|heading| heading.level))
}

fn heading_slug(node: &Node) -> String {
    let mut title = String::new();
    append_title_text(node, &mut title);
    title
        .trim()
        .chars()
        .flat_map(char::to_lowercase)
        .filter_map(slug_char)
        .collect()
}

fn append_title_text(node: &Node, title: &mut String) {
    if let Some(text) = node.cast::<TextSpecial>() {
        title.push_str(&text.content);
    } else if let Some(text) = node.cast::<Text>() {
        title.push_str(&text.content);
    } else if !node.is::<Image>() {
        for child in &node.children {
            append_title_text(child, title);
        }
    }
}

fn slug_char(value: char) -> Option<char> {
    if value == ' ' {
        return Some('-');
    }
    (value.is_alphanumeric() || matches!(value, '-' | '_')).then_some(value)
}

fn unique_slug(slug: String, taken: &mut HashSet<String>) -> String {
    if taken.insert(slug.clone()) {
        return slug;
    }
    (1..)
        .map(|ordinal| format!("{slug}-{ordinal}"))
        .find(|candidate| taken.insert(candidate.clone()))
        .unwrap()
}

struct TaskLists;
impl CoreRule for TaskLists {
    fn run(root: &mut Node, _: &MarkdownIt) {
        root.walk_mut(|node, _| {
            if !node.is::<BulletList>() && !node.is::<OrderedList>() {
                return;
            }
            if mark_task_items(&mut node.children) {
                node.attrs.push(("class", "contains-task-list".into()));
            }
        });
    }
}

fn mark_task_items(items: &mut [Node]) -> bool {
    let mut has_tasks = false;
    for item in items.iter_mut().filter(|item| item.is::<ListItem>()) {
        if add_checkbox(item).is_some() {
            item.attrs.push(("class", "task-list-item".into()));
            has_tasks = true;
        }
    }
    has_tasks
}

fn add_checkbox(item: &mut Node) -> Option<()> {
    // The parser flattens paragraph wrappers in tight lists.
    let inline = if item.children.first()?.is::<Paragraph>() {
        &mut item.children[0].children
    } else {
        &mut item.children
    };
    let text = inline.first_mut()?.cast_mut::<Text>()?;
    let checked = task_state(&text.content)?;
    text.content.drain(..3);
    inline.insert(0, Node::new(Checkbox(checked)));
    Some(())
}

fn task_state(text: &str) -> Option<bool> {
    let rest = text.get(3..)?;
    if !rest.as_bytes().first().is_some_and(u8::is_ascii_whitespace) {
        return None;
    }
    match &text[..3] {
        "[ ]" => Some(false),
        "[x]" | "[X]" => Some(true),
        _ => None,
    }
}

#[derive(Debug)]
struct Checkbox(bool);
impl NodeValue for Checkbox {
    fn render(&self, _: &Node, html: &mut dyn Renderer) {
        let checked = if self.0 { " checked=\"checked\"" } else { "" };
        html.text_raw(&format!("<input class=\"task-list-item-checkbox\"{checked} disabled=\"disabled\" type=\"checkbox\">"));
    }
}
