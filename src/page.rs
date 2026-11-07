use std::sync::LazyLock;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use regex::Regex;

use crate::topic::Topic;
use crate::{FontPreset, PageError};

/// Explicit page appearance overrides. Unset fields use the document's choice.
#[derive(Debug, Default, Clone, Copy)]
pub struct PageOptions {
    pub font: Option<FontPreset>,
}

const TEMPLATE: &str = include_str!("../assets/template.html");
const STYLES: &str = include_str!("../assets/notion.css");
const TERMS_STYLES: &str = "/* terms:start */";
const TERMS_SCRIPT_START: &str = "<!-- terms-script:start -->\n";
const TERMS_SCRIPT_END: &str = "<!-- terms-script:end -->\n";
const IMAGE_PREFIXES: &[&str] = &["http://", "https://", "data:", "./", "../", "/"];
const TAG_COLORS: &[&str] = &[
    "blue", "purple", "green", "orange", "pink", "yellow", "red", "brown",
];
const FAVICON_ENCODING: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'/')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

static PLACEHOLDER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\$|\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());
static DIAGRAM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)(<figure\b[^>]*>\s*)(<svg\b.*?</svg>)").unwrap());

pub(crate) fn render_shell(
    topic: &Topic,
    body_html: &str,
    code_css: &str,
    lang: &str,
    extra_css: &str,
    options: &PageOptions,
    has_terms: bool,
) -> Result<String, PageError> {
    let cover = cover_html(topic.cover())?;
    let font = options.font.or(topic.font()).unwrap_or_default();
    let base_styles = if has_terms {
        STYLES
    } else {
        STYLES
            .split_once(TERMS_STYLES)
            .map_or(STYLES, |(base, _)| base)
    };
    let mut styles = format!("{code_css}\n{base_styles}");
    let font_styles = font.stylesheet();
    if !font_styles.is_empty() {
        styles.push('\n');
        styles.push_str(font_styles);
    }
    if !extra_css.is_empty() {
        styles.push('\n');
        styles.push_str(extra_css);
    }
    let body = DIAGRAM.replace_all(body_html, r#"${1}<div class="diagram-scroll">${2}</div>"#);
    let values: [(&str, &str); 10] = [
        ("lang", &escape_html(lang)),
        ("title", &escape_html(topic.title())),
        ("description", &escape_html(topic.summary())),
        ("favicon", &favicon(topic.icon())),
        ("icon", &escape_html(topic.icon())),
        ("styles", &styles),
        ("font", font.as_str()),
        ("cover", &cover),
        ("properties", &properties_html(topic)),
        ("body", &body),
    ];

    let template = if has_terms {
        std::borrow::Cow::Borrowed(TEMPLATE)
    } else {
        let (before, script) = TEMPLATE
            .split_once(TERMS_SCRIPT_START)
            .expect("template has glossary script marker");
        let (_, after) = script
            .split_once(TERMS_SCRIPT_END)
            .expect("template has glossary script end marker");
        std::borrow::Cow::Owned(format!("{before}{after}"))
    };
    Ok(substitute(&template, &values))
}

fn substitute(template: &str, values: &[(&str, &str)]) -> String {
    let mut result = String::new();
    let mut cursor = 0;
    for captures in PLACEHOLDER.captures_iter(template) {
        let placeholder = captures
            .get(0)
            .expect("regex matches include the whole match");
        result.push_str(&template[cursor..placeholder.start()]);
        let replacement = captures.get(1).map_or("$", |name| {
            values
                .iter()
                .find(|(key, _)| *key == name.as_str())
                .map_or(placeholder.as_str(), |(_, value)| *value)
        });
        result.push_str(replacement);
        cursor = placeholder.end();
    }
    result.push_str(&template[cursor..]);
    result
}

fn cover_html(cover: &str) -> Result<String, PageError> {
    if cover.is_empty() || cover.eq_ignore_ascii_case("yok") {
        return Ok(String::new());
    }
    if IMAGE_PREFIXES
        .iter()
        .any(|prefix| cover.starts_with(prefix))
    {
        let url = escape_html(cover);
        return Ok(format!(
            r#"<div class="cover" style="background-image:url('{url}')"></div>"#
        ));
    }
    if let Some(color) = css_color(cover) {
        return Ok(format!(r#"<div class="cover cover-{color}"></div>"#));
    }
    Err(PageError::Document(format!(
        "Bilinmeyen kapak: '{cover}'. Bir renk adı, görsel adresi ya da 'yok' yazın."
    )))
}

fn favicon(icon: &str) -> String {
    let icon = escape_html(icon);
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>{icon}</text></svg>"
    );
    let encoded = utf8_percent_encode(&svg, FAVICON_ENCODING);
    format!("data:image/svg+xml,{encoded}")
}

fn properties_html(topic: &Topic) -> String {
    let mut rows = String::new();
    if !topic.date().is_empty() {
        rows.push_str(&property_row("📅 Tarih", &escape_html(topic.date())));
    }
    if !topic.tags().is_empty() {
        let tags = topic
            .tags()
            .iter()
            .map(|tag| tag_pill(tag))
            .collect::<Vec<_>>();
        rows.push_str(&property_row("🏷️ Etiketler", &tags.join(" ")));
    }
    if rows.is_empty() {
        return rows;
    }
    format!(r#"<div class="properties">{rows}</div>"#)
}

fn property_row(label: &str, value_html: &str) -> String {
    format!(
        concat!(
            r#"<div class="property">"#,
            r#"<div class="property-label">{}</div>"#,
            r#"<div class="property-value">{}</div>"#,
            "</div>"
        ),
        label, value_html,
    )
}

fn tag_pill(tag: &str) -> String {
    let color = TAG_COLORS[(crc32fast::hash(tag.as_bytes()) as usize) % TAG_COLORS.len()];
    let tag = escape_html(tag);
    format!(r#"<span class="tag bg-{color}">{tag}</span>"#)
}

pub(crate) fn css_color(name: &str) -> Option<&'static str> {
    match name.to_lowercase().as_str() {
        "gri" | "gray" => Some("gray"),
        "kahverengi" | "kahve" | "brown" => Some("brown"),
        "turuncu" | "orange" => Some("orange"),
        "sari" | "sarı" | "yellow" => Some("yellow"),
        "yesil" | "yeşil" | "green" => Some("green"),
        "mavi" | "blue" => Some("blue"),
        "mor" | "purple" => Some("purple"),
        "pembe" | "pink" => Some("pink"),
        "kirmizi" | "kırmızı" | "red" => Some("red"),
        _ => None,
    }
}

pub(crate) fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(topic: &Topic, body_html: &str, code_css: &str) -> Result<String, PageError> {
        render_shell(
            topic,
            body_html,
            code_css,
            "tr",
            "",
            &PageOptions::default(),
            false,
        )
    }

    fn topic(header: &str) -> Topic {
        Topic::parse(&format!("---\n{header}\n---\n")).unwrap().0
    }

    #[test]
    fn metadata_is_escaped_without_expanding_template_tokens() {
        let topic = topic("baslik: \"A < B & $body\"\nozet: 'Merhaba \"dünya\"'");
        let html = render(&topic, "<p>Gövde</p>", "").unwrap();
        assert!(html.contains("<title>A &lt; B &amp; $body</title>"));
        assert!(html.contains("content=\"Merhaba &quot;dünya&quot;\""));
    }

    #[test]
    fn same_tag_uses_python_crc32_color() {
        let topic = topic("baslik: Başlık\netiketler: [token, llm]");
        let html = render(&topic, "", "").unwrap();
        assert!(html.contains("<span class=\"tag bg-blue\">llm</span>"));
    }

    #[test]
    fn scrolls_svg_without_scrolling_caption() {
        let topic = topic("baslik: Başlık");
        let body = "<figure>\n<svg>\n<text>çizim</text></svg>\n<figcaption>Altyazı</figcaption>";
        let html = render(&topic, body, "").unwrap();
        assert!(html.contains("<figure>\n<div class=\"diagram-scroll\"><svg>"));
        assert!(html.contains("</svg></div>\n<figcaption>"));
    }

    #[test]
    fn rejects_unknown_cover() {
        let topic = topic("baslik: Başlık\nkapak: lacivert");
        assert!(
            render(&topic, "", "")
                .unwrap_err()
                .to_string()
                .contains("lacivert")
        );
    }
}
