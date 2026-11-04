use std::sync::LazyLock;

use markdown_it::common::utils::unescape_all;
use markdown_it::plugins::cmark::block::fence::CodeFence;
use markdown_it::{Node, Renderer};
use syntect::html::{ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::page::escape_html;

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

pub(crate) fn code_styles_css() -> &'static str {
    r#"pre code .syntax-comment { color: #3d7b7b; font-style: italic; }
pre code .syntax-keyword, pre code .syntax-storage { color: #008000; font-weight: bold; }
pre code .syntax-string { color: #ba2121; }
pre code .syntax-constant { color: #666; }
pre code .syntax-entity.syntax-function { color: #0000ff; }
@media (prefers-color-scheme: dark) {
  pre code .syntax-comment { color: #8b949e; }
  pre code .syntax-keyword, pre code .syntax-storage { color: #ff7b72; }
  pre code .syntax-string { color: #a5d6ff; }
  pre code .syntax-constant { color: #79c0ff; }
  pre code .syntax-entity.syntax-function { color: #d2a8ff; }
}"#
}

pub(super) fn render_fence(fence: &CodeFence, node: &Node, html: &mut dyn Renderer) {
    let info = unescape_all(&fence.info);
    let language = info.split_whitespace().next().unwrap_or_default();
    let mut attrs = node.attrs.clone();
    if !language.is_empty() {
        attrs.push(("class", format!("{}{}", fence.lang_prefix, language)));
    }
    let content = fence.content.replace("\r\n", "\n");
    html.cr();
    html.open("pre", &[]);
    html.open("code", &attrs);
    html.text_raw(&highlighted(language, &content).unwrap_or_else(|| escape_html(&content)));
    html.close("code");
    html.close("pre");
    html.cr();
}

fn highlighted(language: &str, content: &str) -> Option<String> {
    if language.is_empty() || matches!(language, "text" | "txt" | "plain") {
        return None;
    }
    let syntax = SYNTAXES.find_syntax_by_token(language)?;
    let class_style = ClassStyle::SpacedPrefixed { prefix: "syntax-" };
    let mut generator = ClassedHTMLGenerator::new_with_class_style(syntax, &SYNTAXES, class_style);
    for line in LinesWithEndings::from(content) {
        generator
            .parse_html_for_line_which_includes_newline(line)
            .ok()?;
    }
    Some(generator.finalize())
}
