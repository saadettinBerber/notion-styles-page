//! Glossary syntax and literal matching; rendering lives in the body renderer.

use std::collections::{BTreeMap, HashSet};

use markdown_it::common::sourcemap::SourcePos;
use markdown_it::parser::block::{BlockRule, BlockState};
use markdown_it::parser::inline::InlineRoot;
use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};

use super::blocks::ContainerRule;
use crate::PageError;

#[derive(Debug, Default)]
pub(crate) struct Terms {
    problems: Vec<Problem>,
}

#[derive(Debug)]
struct Problem {
    offset: usize,
    message: String,
}

#[derive(Debug)]
pub(crate) struct TermDefinition {
    pub term: String,
    pub definition: String,
}

impl NodeValue for Terms {
    fn render(&self, _: &Node, _: &mut dyn Renderer) {}
}

impl NodeValue for TermDefinition {
    fn render(&self, node: &Node, renderer: &mut dyn Renderer) {
        renderer.contents(&node.children);
    }
}

pub(super) fn add(parser: &mut MarkdownIt) {
    parser
        .block
        .add_rule::<TermsRule>()
        .before::<ContainerRule>();
}

struct TermsRule;

impl BlockRule for TermsRule {
    fn check(state: &mut BlockState) -> Option<()> {
        opening(state, state.line).map(|_| ())
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        let (fence_size, arguments) = opening(state, state.line)?;
        let start = state.line;
        let start_offset = state.line_offsets[start].first_nonspace;
        let mut terms = Terms::default();
        if !arguments.is_empty() {
            terms.problems.push(Problem {
                offset: start_offset,
                message: ":::terms bloğu ek başlık veya seçenek kabul etmez.".into(),
            });
        }
        let mut definitions = Vec::new();
        let mut line = start + 1;
        let mut closed = false;
        while line < state.line_max {
            let text = state.get_line(line);
            if !text.is_empty() && state.line_indent(line) < 0 {
                break;
            }
            let colons = text.bytes().take_while(|byte| *byte == b':').count();
            if colons >= fence_size && text[colons..].trim().is_empty() {
                closed = true;
                break;
            }
            let offset = state.line_offsets[line].first_nonspace;
            if opening(state, line).is_some() {
                terms.problems.push(Problem {
                    offset,
                    message: "Bir sayfada yalnızca bir :::terms bloğu kullanılabilir.".into(),
                });
            } else if !text.trim().is_empty() {
                match definition(text, offset) {
                    Some(node) => definitions.push(node),
                    None => terms.problems.push(Problem {
                        offset,
                        message: "Terim '- **Terim:** açıklama' biçiminde olmalı.".into(),
                    }),
                }
            }
            line += 1;
        }
        if !closed {
            terms.problems.push(Problem {
                offset: start_offset,
                message: "':::terms' bloğunun kapanış satırı (:::) yok.".into(),
            });
        }
        let mut node = Node::new(terms);
        node.children = definitions;
        Some((node, line - start + usize::from(closed)))
    }
}

fn opening<'a>(state: &'a BlockState, line: usize) -> Option<(usize, &'a str)> {
    if state.line_indent(line) >= state.md.max_indent {
        return None;
    }
    let text = state.get_line(line);
    let fence_size = text.bytes().take_while(|byte| *byte == b':').count();
    if fence_size < 3 {
        return None;
    }
    let info = text[fence_size..].trim();
    let (name, arguments) = info.split_once(char::is_whitespace).unwrap_or((info, ""));
    (name == "terms").then_some((fence_size, arguments.trim()))
}

fn definition(text: &str, offset: usize) -> Option<Node> {
    let after_dash = text.strip_prefix('-')?;
    if !after_dash.starts_with(char::is_whitespace) {
        return None;
    }
    let body = after_dash.trim_start().strip_prefix("**")?;
    let (term, after_term) = body.split_once(":**")?;
    if term.is_empty()
        || term.trim() != term
        || term.contains("**")
        || !after_term.starts_with(char::is_whitespace)
    {
        return None;
    }
    let content = after_term.trim();
    if content.is_empty() {
        return None;
    }
    let definition_offset = offset + (content.as_ptr() as usize - text.as_ptr() as usize);
    let mut node = Node::new(TermDefinition {
        term: term.into(),
        definition: content.into(),
    });
    node.srcmap = Some(SourcePos::new(offset, offset + text.len()));
    let mut inline = Node::new(InlineRoot::new(
        content.into(),
        vec![(0, definition_offset)],
    ));
    inline.srcmap = Some(SourcePos::new(
        definition_offset,
        definition_offset + content.len(),
    ));
    node.children.push(inline);
    Some(node)
}

pub(crate) fn definitions(root: &Node) -> Vec<&Node> {
    fn collect<'a>(node: &'a Node, output: &mut Vec<&'a Node>) {
        if node.is::<TermDefinition>() {
            output.push(node);
        } else {
            for child in &node.children {
                collect(child, output);
            }
        }
    }
    let mut output = Vec::new();
    collect(root, &mut output);
    output
}

pub(crate) fn validate(root: &Node, source: &str, body_start: usize) -> Result<(), PageError> {
    let mut blocks = 0;
    let mut names = HashSet::new();
    let mut problems = Vec::new();
    root.walk(|node, _| {
        let offset = node.srcmap.map_or(0, |span| span.get_byte_offsets().0);
        if let Some(terms) = node.cast::<Terms>() {
            blocks += 1;
            if blocks > 1 {
                problems.push((
                    offset,
                    "Bir sayfada yalnızca bir :::terms bloğu kullanılabilir.".into(),
                ));
            }
            problems.extend(
                terms
                    .problems
                    .iter()
                    .map(|problem| (problem.offset, problem.message.clone())),
            );
        } else if let Some(term) = node.cast::<TermDefinition>()
            && !names.insert(term.term.as_str())
        {
            problems.push((offset, format!("Yinelenen terim: '{}'.", term.term)));
        }
    });
    let Some((offset, message)) = problems.into_iter().min_by_key(|problem| problem.0) else {
        return Ok(());
    };
    let bytes = source.as_bytes();
    let line = bytes
        .iter()
        .take(body_start + offset)
        .enumerate()
        .filter(|&(index, byte)| {
            *byte == b'\n' || (*byte == b'\r' && bytes.get(index + 1) != Some(&b'\n'))
        })
        .count()
        + 1;
    Err(PageError::Document(format!("Satır {line}: {message}")))
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Match {
    pub start: usize,
    pub end: usize,
    pub term_index: usize,
}

/// Literal matches with original byte offsets, suppressing repeated terms in a section.
#[cfg(test)]
pub(crate) fn matches(text: &str, terms: &[String], used: &mut HashSet<usize>) -> Vec<Match> {
    matches_after(text, terms, used, None)
}

/// `previous` preserves the word boundary when inline formatting splits text nodes.
pub(crate) fn matches_after(
    text: &str,
    terms: &[String],
    used: &mut HashSet<usize>,
    previous: Option<char>,
) -> Vec<Match> {
    let variants: Vec<_> = terms.iter().map(|term| lowercase_first(term)).collect();
    let lengths: Vec<_> = terms.iter().map(|term| term.chars().count()).collect();
    let mut candidates = Vec::new();
    let mut preceding = previous;
    for (position, character) in text.char_indices() {
        let rest = &text[position..];
        let at_boundary = !preceding.is_some_and(char::is_alphanumeric);
        preceding = Some(character);
        if !at_boundary {
            continue;
        }
        for (index, term) in terms.iter().enumerate() {
            if term.is_empty() {
                continue;
            }
            let length = if rest.starts_with(term) {
                Some(term.len())
            } else {
                variants[index]
                    .as_ref()
                    .filter(|variant| rest.starts_with(variant.as_str()))
                    .map(String::len)
            };
            if let Some(length) = length {
                candidates.push((
                    lengths[index],
                    Match {
                        start: position,
                        end: suffix_end(text, position + length),
                        term_index: index,
                    },
                ));
            }
        }
    }
    candidates.sort_unstable_by(|(left_length, left), (right_length, right)| {
        right_length
            .cmp(left_length)
            .then_with(|| left.start.cmp(&right.start))
            .then_with(|| left.term_index.cmp(&right.term_index))
    });
    let mut selected: BTreeMap<usize, Match> = BTreeMap::new();
    for (_, candidate) in candidates {
        let overlaps_before = selected
            .range(..=candidate.start)
            .next_back()
            .is_some_and(|(_, found)| found.end > candidate.start);
        let overlaps_after = selected
            .range(candidate.start..)
            .next()
            .is_some_and(|(_, found)| found.start < candidate.end);
        if !overlaps_before && !overlaps_after {
            selected.insert(candidate.start, candidate);
        }
    }
    // Already-used terms still reserve their spans, so shorter overlapping terms
    // cannot steal a repeated occurrence of a longer term.
    selected
        .into_values()
        .filter(|found| used.insert(found.term_index))
        .collect()
}

fn lowercase_first(term: &str) -> Option<String> {
    let mut letters = term.chars();
    let first = letters.next()?;
    if !letters.next()?.is_lowercase() {
        return None;
    }
    let lower = match first {
        'İ' => "i".into(),
        'I' => "ı".into(),
        character => character.to_lowercase().collect::<String>(),
    };
    Some(format!("{lower}{}", &term[first.len_utf8()..]))
}

fn suffix_end(text: &str, mut end: usize) -> usize {
    let mut remaining = text[end..].chars().peekable();
    if remaining
        .peek()
        .is_some_and(|character| matches!(character, '\'' | '’'))
    {
        let apostrophe = remaining.next().unwrap();
        if remaining
            .peek()
            .is_some_and(|character| character.is_alphabetic())
        {
            end += apostrophe.len_utf8();
        } else {
            return end;
        }
    }
    for character in remaining.take_while(|character| character.is_alphabetic()) {
        end += character.len_utf8();
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> MarkdownIt {
        let mut parser = MarkdownIt::new();
        markdown_it::plugins::cmark::add(&mut parser);
        super::super::blocks::add_reader(&mut parser);
        add(&mut parser);
        parser
    }

    #[test]
    fn parses_inline_definitions_and_preserves_source_ranges() {
        let source = ":::terms\r\n- **BREX:** A `code` and *word* [link](https://example.test).\r\n- **Veri modülü:** Bilgi.\r\n:::\r\n";
        let root = parser().parse(source);
        validate(&root, source, 0).unwrap();
        let entries = definitions(&root);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].cast::<TermDefinition>().unwrap().term, "BREX");
        assert_eq!(entries[0].collect_text(), "A code and word link.");
        let (start, end) = entries[1].srcmap.unwrap().get_byte_offsets();
        assert_eq!(&source[start..end], "- **Veri modülü:** Bilgi.");
        assert_eq!(root.render(), "");
    }

    #[test]
    fn rejects_malformed_duplicate_and_second_blocks_at_source_lines() {
        let header = "---\r\ntitle: Test\r\n---\r\n";
        for (body, line, problem) in [
            (
                ":::terms\r\n- **BREX** definition\r\n:::\r\n",
                5,
                "biçiminde",
            ),
            (
                ":::terms\r\n- **BREX:** One\r\n- **BREX:** Two\r\n:::\r\n",
                6,
                "Yinelenen",
            ),
            (
                ":::terms\r\n- **BREX:** One\r\n:::\r\n\r\n:::terms\r\n- **XSD:** Two\r\n:::\r\n",
                8,
                "yalnızca bir",
            ),
        ] {
            let source = format!("{header}{body}");
            let error = validate(&parser().parse(body), &source, header.len())
                .unwrap_err()
                .to_string();
            assert!(error.starts_with(&format!("Satır {line}:")), "{error}");
            assert!(error.contains(problem), "{error}");
        }
    }

    #[test]
    fn ignores_fenced_code_and_finds_nested_glossary() {
        let source = "```text\n:::terms\n- invalid\n:::\n```\n\n:::callout\n:::terms\n- **BREX:** Definition\n:::\n:::\n";
        let root = parser().parse(source);
        validate(&root, source, 0).unwrap();
        super::super::blocks::validate(&root, source, 0).unwrap();
        assert_eq!(definitions(&root).len(), 1);

        let mut page_parser = MarkdownIt::new();
        markdown_it::plugins::cmark::add(&mut page_parser);
        super::super::blocks::add(&mut page_parser);
        add(&mut page_parser);
        let page_root = page_parser.parse(source);
        validate(&page_root, source, 0).unwrap();
        super::super::blocks::validate(&page_root, source, 0).unwrap();
        assert_eq!(definitions(&page_root).len(), 1);
    }

    #[test]
    fn reference_links_share_document_context_and_quote_lines_keep_offsets() {
        let body = "> :::terms\r\n> - **BREX:** [Rules][rules]\r\n> :::\r\n\r\n[rules]: https://example.test\r\n";
        let root = parser().parse(body);
        validate(&root, body, 0).unwrap();
        let entries = definitions(&root);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].render(),
            "<a href=\"https://example.test\">Rules</a>"
        );
        let (start, end) = entries[0].srcmap.unwrap().get_byte_offsets();
        assert_eq!(&body[start..end], "- **BREX:** [Rules][rules]");
        let invalid = body.replace("- **BREX:**", "- **BREX**");
        let error = validate(&parser().parse(&invalid), &invalid, 0)
            .unwrap_err()
            .to_string();
        assert!(error.starts_with("Satır 2:"), "{error}");
    }

    #[test]
    fn rejects_empty_items_arguments_and_unclosed_block() {
        for source in [
            ":::terms\n- **:** Empty\n:::\n",
            ":::terms\n- **BREX:**\n:::\n",
            ":::terms title\n- **BREX:** Definition\n:::\n",
            ":::terms\n- **BREX:** Definition\n",
        ] {
            assert!(validate(&parser().parse(source), source, 0).is_err());
        }
    }

    #[test]
    fn reports_nested_second_block_and_unclosed_block_at_original_lines() {
        let header = "\u{feff}---\r\ntitle: Başlık\r\n---\r\n";
        for (body, expected_line, expected_message) in [
            (
                "Ön söz\r\n\r\n:::terms\r\n- **XSD:** Bir\r\n:::terms\r\n- **BREX:** İki\r\n:::\r\n:::\r\n",
                8,
                "yalnızca bir",
            ),
            ("Ön söz\r\n\r\n:::terms\r\n- **XSD:** Bir\r\n", 6, "kapanış"),
        ] {
            let source = format!("{header}{body}");
            let error = validate(&parser().parse(body), &source, header.len())
                .unwrap_err()
                .to_string();
            assert!(
                error.starts_with(&format!("Satır {expected_line}:")),
                "{error}"
            );
            assert!(error.contains(expected_message), "{error}");
        }
    }

    fn marked<'a>(text: &'a str, terms: &[&str]) -> Vec<(usize, &'a str)> {
        matches(
            text,
            &terms.iter().map(|term| (*term).into()).collect::<Vec<_>>(),
            &mut HashSet::new(),
        )
        .iter()
        .map(|found| (found.term_index, &text[found.start..found.end]))
        .collect()
    }

    #[test]
    fn includes_suffixes_and_marks_only_first_occurrence() {
        assert_eq!(marked("BREX'e BREX’ten BREX", &["BREX"]), [(0, "BREX'e")]);
        assert_eq!(marked("XSD’nin XSD", &["XSD"]), [(0, "XSD’nin")]);
        assert_eq!(
            marked("veri modülünü", &["veri modülü"]),
            [(0, "veri modülünü")]
        );
        assert_eq!(marked("BREX' ", &["BREX"]), [(0, "BREX")]);
    }

    #[test]
    fn lowers_only_the_first_letter_with_turkish_mapping() {
        assert_eq!(marked("crate'leri", &["Crate"]), [(0, "crate'leri")]);
        assert_eq!(
            marked("işlem ısı", &["İşlem", "Isı"]),
            [(0, "işlem"), (1, "ısı")]
        );
        assert!(marked("brex xsd", &["BREX", "XSD"]).is_empty());
        assert!(marked("CRATE", &["Crate"]).is_empty());
    }

    #[test]
    fn uses_unicode_start_boundaries_and_longest_overlap() {
        assert_eq!(marked("aBREX 2BREX öBREX (BREX)", &["BREX"]), [(0, "BREX")]);
        assert_eq!(
            marked("veri modülünü, veri modülü, veri", &["veri", "veri modülü"]),
            [(1, "veri modülünü"), (0, "veri")]
        );
    }

    #[test]
    fn preserves_start_boundaries_across_inline_nodes() {
        let terms = vec!["BREX".into()];
        assert!(matches_after("BREX", &terms, &mut HashSet::new(), Some('a')).is_empty());
        assert_eq!(
            matches_after("BREX", &terms, &mut HashSet::new(), Some(' ')).len(),
            1
        );
    }

    #[test]
    fn longer_crossing_match_wins_even_when_it_starts_later() {
        assert_eq!(
            marked(
                "alpha beta gamma delta; alpha beta",
                &["alpha beta", "beta gamma delta"]
            ),
            [(1, "beta gamma delta"), (0, "alpha beta")]
        );
        let terms = vec!["alpha beta".into(), "beta gamma delta".into()];
        let mut used = HashSet::from([1]);
        assert!(matches_after("alpha beta gamma delta", &terms, &mut used, None).is_empty());
        assert_eq!(used, HashSet::from([1]));
    }
}
