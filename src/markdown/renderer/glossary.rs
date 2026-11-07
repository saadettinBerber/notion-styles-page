use std::collections::HashSet;
use std::sync::LazyLock;

use markdown_it::Node;
use markdown_it::common::utils::{escape_html, unescape_all};
use markdown_it::parser::inline::{Text, TextSpecial};
use markdown_it::plugins::cmark::block::code::CodeBlock;
use markdown_it::plugins::cmark::block::fence::CodeFence;
use markdown_it::plugins::cmark::block::heading::ATXHeading;
use markdown_it::plugins::cmark::block::lheading::SetextHeader;
use markdown_it::plugins::cmark::block::list::ListItem;
use markdown_it::plugins::cmark::block::paragraph::Paragraph;
use markdown_it::plugins::cmark::inline::backticks::CodeInline;
use markdown_it::plugins::cmark::inline::image::Image;
use markdown_it::plugins::cmark::inline::link::Link;
use markdown_it::plugins::cmark::inline::newline::{Hardbreak, Softbreak};
use markdown_it::plugins::extra::tables::TableCell;
use markdown_it::plugins::html::html_block::HtmlBlock;
use markdown_it::plugins::html::html_inline::HtmlInline;
use regex::Regex;

use super::super::blocks::{Container, Summary};
use super::super::terms::{self, TermDefinition, Terms};

static HTML_TAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)<!--.*?(?:-->|$)|<!\[CDATA\[.*?(?:\]\]>|$)|<\?.*?(?:\?>|$)|</?([A-Za-z][A-Za-z0-9:-]*)(?:[^<>"']|"[^"]*"|'[^']*')*>"#).unwrap()
});

#[derive(Default)]
pub(super) struct Glossary {
    pub(super) names: Vec<String>,
    pub(super) ids: Vec<String>,
    used: HashSet<usize>,
    used_anywhere: HashSet<usize>,
    prose: bool,
    blocked: bool,
    previous: Option<char>,
    html_tags: Vec<String>,
}

pub(super) struct Scope {
    prose: bool,
    blocked: bool,
    boundary: Option<Option<char>>,
}

impl Glossary {
    pub(super) fn new(root: &Node, prefix: &str) -> Self {
        let names: Vec<_> = terms::definitions(root)
            .iter()
            .map(|node| node.cast::<TermDefinition>().unwrap().term.clone())
            .collect();
        let mut taken = HashSet::new();
        root.walk(|node, _| {
            for (_, id) in node.attrs.iter().filter(|(key, _)| *key == "id") {
                taken.insert(format!("{prefix}{id}"));
            }
            let raw = node
                .cast::<HtmlBlock>()
                .map(|html| html.content.as_str())
                .or_else(|| node.cast::<HtmlInline>().map(|html| html.content.as_str()));
            if let Some(raw) = raw {
                for tag in HTML_TAG
                    .captures_iter(raw)
                    .filter(|tag| tag.get(1).is_some())
                {
                    if let Some(id) = html_id(tag.get(0).unwrap().as_str()) {
                        taken.insert(id);
                    }
                }
            }
        });
        let ids = names
            .iter()
            .map(|name| {
                let base = format!("{prefix}term-{}", slug(name));
                let mut id = base.clone();
                let mut ordinal = 2;
                while !taken.insert(id.clone()) {
                    id = format!("{base}-{ordinal}");
                    ordinal += 1;
                }
                id
            })
            .collect();
        Self {
            names,
            ids,
            ..Self::default()
        }
    }

    pub(super) fn enter(&mut self, node: &Node) -> Scope {
        let level = node
            .cast::<ATXHeading>()
            .map(|h| h.level)
            .or_else(|| node.cast::<SetextHeader>().map(|h| h.level));
        if level == Some(2) && !self.blocked && self.html_tags.is_empty() {
            self.used.clear();
        }
        let starts_prose =
            node.is::<Paragraph>() || node.is::<ListItem>() || node.is::<TableCell>();
        let scope = Scope {
            prose: self.prose,
            blocked: self.blocked,
            boundary: starts_prose.then(|| self.previous.take()),
        };
        self.prose |= starts_prose;
        self.blocked |= level.is_some()
            || node.is::<Summary>()
            || node.is::<CodeInline>()
            || node.is::<CodeBlock>()
            || node.is::<CodeFence>()
            || node.is::<Link>()
            || node.is::<Image>()
            || node.is::<HtmlBlock>()
            || node.is::<Terms>()
            || node.is::<TermDefinition>()
            || node
                .cast::<Container>()
                .is_some_and(|block| block.name() == "video");
        if let Some(html) = node.cast::<HtmlInline>() {
            self.track_html(&html.content);
        } else if let Some(html) = node.cast::<HtmlBlock>() {
            self.track_html(&html.content);
        }
        if node.is::<Softbreak>() || node.is::<Hardbreak>() {
            self.previous = Some(' ');
        }
        scope
    }

    pub(super) fn leave(&mut self, scope: Scope) {
        self.prose = scope.prose;
        self.blocked = scope.blocked;
        if let Some(previous) = scope.boundary {
            self.previous = previous;
        }
    }

    pub(super) fn text_node(&mut self, node: &Node, output: &mut String) -> bool {
        let text = if let Some(text) = node.cast::<Text>() {
            &text.content
        } else if let Some(text) = node.cast::<TextSpecial>() {
            &text.content
        } else {
            return false;
        };
        if !self.names.is_empty() && self.prose && !self.blocked && self.html_tags.is_empty() {
            let matches = terms::matches_after(text, &self.names, &mut self.used, self.previous);
            let mut cursor = 0;
            for found in matches {
                output.push_str(&escape_html(&text[cursor..found.start]));
                output.push_str("<button type=\"button\" class=\"term\" popovertarget=\"");
                output.push_str(&escape_html(&self.ids[found.term_index]));
                output.push_str("\">");
                output.push_str(&escape_html(&text[found.start..found.end]));
                output.push_str("</button>");
                cursor = found.end;
                self.used_anywhere.insert(found.term_index);
            }
            output.push_str(&escape_html(&text[cursor..]));
            self.previous = text.chars().next_back().or(self.previous);
            true
        } else {
            if self.prose {
                self.previous = text.chars().next_back().or(self.previous);
            }
            false
        }
    }

    pub(super) fn has_inline_prose(&self) -> bool {
        !self.names.is_empty() && self.prose && !self.blocked
    }

    pub(super) fn match_run(&mut self, text: &str) -> Vec<terms::Match> {
        let matches = if self.has_inline_prose() && self.html_tags.is_empty() {
            terms::matches_after(text, &self.names, &mut self.used, self.previous)
        } else {
            Vec::new()
        };
        self.used_anywhere
            .extend(matches.iter().map(|found| found.term_index));
        self.previous = text.chars().next_back().or(self.previous);
        matches
    }

    pub(super) fn warnings(&self) -> Vec<String> {
        self.names
            .iter()
            .enumerate()
            .filter(|(index, _)| !self.used_anywhere.contains(index))
            .map(|(_, name)| format!("Kullanılmayan terim (unused term): '{name}'."))
            .collect()
    }

    fn track_html(&mut self, html: &str) {
        for tag in HTML_TAG.captures_iter(html) {
            let Some(name) = tag.get(1) else {
                continue;
            };
            self.track_tag(
                tag.get(0).unwrap().as_str(),
                &name.as_str().to_ascii_lowercase(),
            );
        }
    }

    fn track_tag(&mut self, html: &str, name: &str) {
        let closing = html.starts_with("</");
        if let Some(raw) = self
            .html_tags
            .last()
            .filter(|tag| matches!(tag.as_str(), "script" | "style" | "textarea" | "title"))
            && (!closing || raw != name)
        {
            return;
        }
        if closing {
            if let Some(index) = self.html_tags.iter().rposition(|open| open == name) {
                self.html_tags.truncate(index);
            }
        } else if !html.trim_end().ends_with("/>")
            && !matches!(
                name,
                "area"
                    | "base"
                    | "br"
                    | "col"
                    | "embed"
                    | "hr"
                    | "img"
                    | "input"
                    | "link"
                    | "meta"
                    | "param"
                    | "source"
                    | "track"
                    | "wbr"
            )
        {
            self.html_tags.push(name.to_owned());
        }
    }
}

/// Read an attribute from an already-tokenized raw tag, respecting quoted values.
fn html_id(tag: &str) -> Option<String> {
    let mut rest = tag.strip_prefix('<')?;
    if rest.starts_with('/') {
        return None;
    }
    rest = rest.trim_start_matches(|character: char| {
        character.is_ascii_alphanumeric() || matches!(character, ':' | '-')
    });
    loop {
        rest = rest.trim_start();
        if rest.is_empty() || rest.starts_with(['>', '/']) {
            return None;
        }
        let name_end = rest
            .find(|character: char| {
                character.is_whitespace() || matches!(character, '=' | '>' | '/')
            })
            .unwrap_or(rest.len());
        let name = &rest[..name_end];
        rest = rest[name_end..].trim_start();
        if !rest.starts_with('=') {
            continue;
        }
        rest = rest[1..].trim_start();
        let quote = rest.chars().next()?;
        let value;
        if matches!(quote, '\'' | '"') {
            let quoted = &rest[1..];
            let end = quoted.find(quote)?;
            value = &quoted[..end];
            rest = &quoted[end + 1..];
        } else {
            let end = rest
                .find(|character: char| character.is_whitespace() || character == '>')
                .unwrap_or(rest.len());
            value = &rest[..end];
            rest = &rest[end..];
        }
        if name.eq_ignore_ascii_case("id") {
            return Some(unescape_all(value).into_owned());
        }
    }
}

fn slug(term: &str) -> String {
    let folded: String = term
        .chars()
        .map(|c| match c {
            'ç' | 'Ç' => 'c',
            'ğ' | 'Ğ' => 'g',
            'ı' | 'İ' => 'i',
            'ö' | 'Ö' => 'o',
            'ş' | 'Ş' => 's',
            'ü' | 'Ü' => 'u',
            c if c.is_ascii_alphanumeric() => c.to_ascii_lowercase(),
            _ => '-',
        })
        .collect();
    let result = folded
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if result.is_empty() {
        "term".into()
    } else {
        result
    }
}
