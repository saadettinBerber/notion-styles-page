use markdown_it::common::sourcemap::SourcePos;
use markdown_it::parser::block::{BlockRule, BlockState};
use markdown_it::parser::extset::MarkdownItExt;
use markdown_it::parser::inline::InlineRoot;
use markdown_it::plugins::cmark::block::fence::FenceScanner;
use markdown_it::{MarkdownIt, Node, NodeValue};

use crate::page::{css_color, escape_html};
use crate::{MediaSource, PageError};

pub(super) const SUMMARY_OPEN: &str = "<summary>";
pub(super) const SUMMARY_CLOSE: &str = "</summary>";
pub(super) const TOGGLE_BODY_OPEN: &str = "<div class=\"toggle-body\">\n";
pub(super) const TOGGLE_BODY_CLOSE: &str = "</div>";
pub(super) const TOGGLE_CLOSE: &str = "</details>\n";

#[derive(Debug)]
pub(super) struct Block<'a> {
    kind: Kind,
    arguments: &'a str,
}

#[derive(Debug)]
pub(super) struct Container {
    kind: Kind,
    arguments: String,
    unclosed: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Callout,
    Toggle,
    ToggleHeading,
    Columns,
    Column,
    Video,
}

impl NodeValue for Container {}

#[derive(Debug)]
pub(super) struct Summary;
impl NodeValue for Summary {}

pub(super) struct ContainerRule;

#[derive(Debug)]
struct ReaderContainers;
impl MarkdownItExt for ReaderContainers {}

pub(super) fn add(parser: &mut MarkdownIt) {
    parser
        .block
        .add_rule::<ContainerRule>()
        .before::<FenceScanner>();
}

pub(super) fn add_reader(parser: &mut MarkdownIt) {
    add(parser);
    parser.ext.insert(ReaderContainers);
    parser
        .block
        .add_rule::<UnknownContainerRule>()
        .after::<ContainerRule>()
        .before::<FenceScanner>();
}

impl BlockRule for ContainerRule {
    fn check(state: &mut BlockState) -> Option<()> {
        opening(state).map(|_| ())
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let (block, fence_size) = opening(state)?;
        let start = state.line;
        let mut node = block.to_node(
            state.line_offsets[state.line].first_nonspace,
            state.get_line(state.line),
        );
        let (end, closed) = find_end(state, fence_size);
        if !closed {
            node.cast_mut::<Container>()?.unclosed = Some(state.get_line(start).trim().into());
        }
        parse_children(state, &mut node, end);
        Some((node, end - start + usize::from(closed)))
    }
}

fn opening<'s>(state: &'s BlockState) -> Option<(Block<'s>, usize)> {
    let (info, fence_size) = container_info(state)?;
    let (name, arguments) = info.split_once(' ').unwrap_or((info, &info[info.len()..]));
    let kind = Kind::named(name)?;
    Some((
        Block {
            kind,
            arguments: arguments.trim(),
        },
        fence_size,
    ))
}

fn container_info<'s>(state: &'s BlockState) -> Option<(&'s str, usize)> {
    if state.line_indent(state.line) >= state.md.max_indent {
        return None;
    }
    let line = state.get_line(state.line);
    let fence_size = colon_count(line);
    let info = line.get(fence_size..)?.trim();
    if fence_size < 3 {
        return None;
    }
    Some((info, fence_size))
}

fn colon_count(line: &str) -> usize {
    line.bytes().take_while(|&byte| byte == b':').count()
}

fn find_end(state: &mut BlockState, fence_size: usize) -> (usize, bool) {
    let balanced = state.md.ext.contains::<ReaderContainers>();
    let mut markers = vec![fence_size];
    let mut line = state.line + 1;
    while line < state.line_max {
        let text = state.get_line(line);
        let indentation = state.line_indent(line);
        if !text.is_empty() && indentation < 0 {
            break;
        }
        if indentation < state.md.max_indent && count_marker(text, &mut markers, balanced) {
            return (line, true);
        }
        line += fenced_lines(state, line).unwrap_or(1);
    }
    (line, false)
}

fn count_marker(text: &str, markers: &mut Vec<usize>, balanced: bool) -> bool {
    if closing_fence(
        text,
        *markers.last().expect("the outer container is still open"),
    ) {
        markers.pop();
        return markers.is_empty();
    }
    let count = colon_count(text);
    let name = text[count..].trim().split(' ').next().unwrap_or_default();
    if count >= 3 && (name == "terms" || (balanced && Kind::named(name).is_some())) {
        markers.push(count);
    }
    false
}

fn closing_fence(text: &str, fence_size: usize) -> bool {
    let count = colon_count(text);
    count >= fence_size && text[count..].trim_matches([' ', '\t']).is_empty()
}

fn fenced_lines(state: &mut BlockState, line: usize) -> Option<usize> {
    let saved_line = std::mem::replace(&mut state.line, line);
    let result = FenceScanner::run(state).map(|(_, count)| count);
    state.line = saved_line;
    result
}

fn parse_children(state: &mut BlockState, node: &mut Node, end: usize) {
    let previous = std::mem::take(node);
    let parent = std::mem::replace(&mut state.node, previous);
    let parent_end = std::mem::replace(&mut state.line_max, end);
    let start = state.line;
    let tight = state.tight;
    state.line += 1;
    state.level += 1;
    state.md.block.tokenize(state);
    state.level -= 1;
    state.line = start;
    state.line_max = parent_end;
    state.tight = tight;
    *node = std::mem::replace(&mut state.node, parent);
}

impl Kind {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "callout" => Self::Callout,
            "toggle" => Self::Toggle,
            "toggle-heading" => Self::ToggleHeading,
            "columns" => Self::Columns,
            "column" => Self::Column,
            "video" => Self::Video,
            _ => return None,
        })
    }

    fn toggle_class(self) -> Option<&'static str> {
        match self {
            Self::Toggle => Some("toggle"),
            Self::ToggleHeading => Some("toggle toggle-heading"),
            _ => None,
        }
    }
}

impl Block<'_> {
    fn to_node(&self, start: usize, line: &str) -> Node {
        let mut container = Node::new(Container {
            kind: self.kind,
            arguments: self.arguments.to_owned(),
            unclosed: None,
        });
        if self.kind.toggle_class().is_some() {
            let mut summary = Node::new(Summary);
            let arguments_start = self.arguments.as_ptr() as usize - line.as_ptr() as usize;
            let start = start + arguments_start;
            summary.srcmap = Some(SourcePos::new(start, start + self.arguments.len()));
            summary.children.push(Node::new(InlineRoot::new(
                self.arguments.to_owned(),
                vec![(0, start)],
            )));
            container.children.push(summary);
        }
        container
    }
}

#[derive(Debug)]
struct UnknownContainer(String);
impl NodeValue for UnknownContainer {}

struct UnknownContainerRule;
impl BlockRule for UnknownContainerRule {
    fn check(state: &mut BlockState) -> Option<()> {
        container_info(state).map(|_| ())
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let (info, _) = container_info(state)?;
        let name = info.split(' ').next().unwrap_or_default();
        Some((Node::new(UnknownContainer(name.to_owned())), 1))
    }
}

pub(super) fn validate(root: &Node, source: &str, body_start: usize) -> Result<(), PageError> {
    let Some((node, problem)) = first_container_problem(root) else {
        return Ok(());
    };
    let relative_start = node.srcmap.map_or(0, |span| span.get_byte_offsets().0);
    let line = original_line(source, body_start + relative_start);
    Err(PageError::Document(format!("Satır {line}: {problem}")))
}

fn first_container_problem(node: &Node) -> Option<(&Node, String)> {
    if let Some(unknown) = node.cast::<UnknownContainer>() {
        return Some((node, format!("bilinmeyen kapsayıcı '{}'.", unknown.0)));
    }
    if let Some(problem) = node.children.iter().find_map(first_container_problem) {
        return Some(problem);
    }
    let line = node.cast::<Container>()?.unclosed.as_ref()?;
    Some((node, format!("'{line}' bloğunun kapanış satırı (:::) yok.")))
}

fn original_line(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    bytes
        .iter()
        .take(start)
        .enumerate()
        .filter(|&(index, byte)| {
            *byte == b'\n' || (*byte == b'\r' && bytes.get(index + 1) != Some(&b'\n'))
        })
        .count()
        + 1
}

impl Container {
    pub(super) fn name(&self) -> &'static str {
        match self.kind {
            Kind::Callout => "callout",
            Kind::Toggle => "toggle",
            Kind::ToggleHeading => "toggle-heading",
            Kind::Columns => "columns",
            Kind::Column => "column",
            Kind::Video => "video",
        }
    }

    pub(super) fn arguments(&self) -> &str {
        &self.arguments
    }

    pub(super) fn is_toggle(&self) -> bool {
        self.kind.toggle_class().is_some()
    }

    pub(super) fn opening(&self, media: &(impl MediaSource + ?Sized)) -> Result<String, PageError> {
        Ok(match self.kind {
            Kind::Callout => callout(self.arguments()),
            Kind::Toggle | Kind::ToggleHeading => {
                format!("<details class=\"{}\">", self.kind.toggle_class().unwrap())
            }
            Kind::Columns => "<div class=\"columns\">\n".into(),
            Kind::Column => "<div class=\"column\">\n".into(),
            Kind::Video => video(self.arguments(), media)?,
        })
    }

    pub(super) fn closing(&self) -> &'static str {
        match self.kind {
            Kind::Callout => "</div></div>\n",
            Kind::Toggle | Kind::ToggleHeading => "</div></details>\n",
            Kind::Columns | Kind::Column => "</div>\n",
            Kind::Video => "</figcaption></figure>\n",
        }
    }
}

fn callout(arguments: &str) -> String {
    let words = arguments.split_whitespace();
    let color = words.clone().find_map(css_color).unwrap_or("gray");
    let icon = words
        .into_iter()
        .find(|word| css_color(word).is_none())
        .unwrap_or("💡");
    format!(
        "<div class=\"callout bg-{color}\"><div class=\"callout-icon\">{}</div><div class=\"callout-body\">\n",
        escape_html(icon)
    )
}

fn video(arguments: &str, media: &(impl MediaSource + ?Sized)) -> Result<String, PageError> {
    if arguments.is_empty() {
        return Err(PageError::Document(
            "Video bloğunda video yolu eksik: ':::video medya/ad.mp4' yazın.".into(),
        ));
    }
    let (path, poster_path) = arguments
        .split_once(char::is_whitespace)
        .unwrap_or((arguments, ""));
    let source = escape_html(&media.video_data_uri(path)?);
    let poster = poster_attribute(poster_path.trim_start(), media)?;
    Ok(format!(
        "<figure class=\"video\"><video controls playsinline preload=\"metadata\"{poster} src=\"{source}\"></video><figcaption>\n"
    ))
}

fn poster_attribute(path: &str, media: &(impl MediaSource + ?Sized)) -> Result<String, PageError> {
    if path.is_empty() {
        return Ok(String::new());
    }
    let source = escape_html(&media.image_data_uri(path)?);
    Ok(format!(" poster=\"{source}\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_reader_rejects_unclosed_and_unknown_markers_at_original_lines() {
        let mut reader = MarkdownIt::new();
        markdown_it::plugins::cmark::add(&mut reader);
        add_reader(&mut reader);
        let header = "\u{feff}---\r\nbaslik: Başlık\r\n---\r\n";
        for (body, expected) in [
            (
                ":::toggle Başlık\r\nGövde\r\n",
                "Satır 4: ':::toggle Başlık' bloğunun kapanış satırı (:::) yok.",
            ),
            (
                "> :::unknown arg\r\n> Gövde\r\n",
                "Satır 4: bilinmeyen kapsayıcı 'unknown'.",
            ),
            (
                "Text\r\n:::unknown\r\n",
                "Satır 5: bilinmeyen kapsayıcı 'unknown'.",
            ),
        ] {
            let source = format!("{header}{body}");
            let root = reader.parse(body);
            assert_eq!(
                validate(&root, &source, header.len())
                    .unwrap_err()
                    .to_string(),
                expected
            );
        }
        let fenced = "```text\r\n:::unknown\r\n```\r\n";
        assert!(validate(&reader.parse(fenced), fenced, 0).is_ok());
    }

    #[test]
    fn strict_reader_balances_equal_nested_fences() {
        let mut parser = MarkdownIt::new();
        markdown_it::plugins::cmark::add(&mut parser);
        add_reader(&mut parser);
        let source = ":::columns\n:::column\nText\n:::\n:::\n";
        let document = parser.parse(source);

        assert!(validate(&document, source, 0).is_ok());
        let outer = document.children.first().unwrap();
        assert_eq!(outer.cast::<Container>().unwrap().name(), "columns");
        assert_eq!(
            outer.children[0].cast::<Container>().unwrap().name(),
            "column"
        );
        assert_eq!(outer.children.len(), 1);
    }
}
