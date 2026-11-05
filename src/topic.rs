use serde_yaml_ng::{Mapping, Value};

use crate::{FontPreset, PageError};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Topic {
    title: String,
    icon: String,
    cover: String,
    date: String,
    tags: Vec<String>,
    summary: String,
    #[serde(default)]
    font: Option<FontPreset>,
}

impl Topic {
    pub(crate) fn parse(source: &str) -> Result<(Self, &str), PageError> {
        let (header, body) = split_header(source)?;
        let fields = parse_fields(header)?;
        let topic = Self::from_fields(&fields)?;
        if topic.title.is_empty() {
            return Err(document_error(
                "Başlık bloğunda 'baslik:' veya 'title:' alanı zorunlu.",
            ));
        }
        Ok((topic, body))
    }

    pub(crate) fn from_fields(fields: &Mapping) -> Result<Self, PageError> {
        let title_key = if fields.contains_key("baslik") {
            "baslik"
        } else {
            "title"
        };
        let title = field_text(fields, title_key);
        let tags = parse_tags(fields.get("etiketler"))?;
        let icon = field_text(fields, "ikon");
        Ok(Self {
            title,
            icon: if icon.is_empty() { "📄".into() } else { icon },
            cover: field_text(fields, "kapak"),
            date: field_text(fields, "tarih"),
            tags,
            summary: field_text(fields, "ozet"),
            font: parse_font(fields.get("font"))?,
        })
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) fn icon(&self) -> &str {
        &self.icon
    }

    pub(crate) fn cover(&self) -> &str {
        &self.cover
    }

    pub(crate) fn date(&self) -> &str {
        &self.date
    }

    pub(crate) fn tags(&self) -> &[String] {
        &self.tags
    }

    pub(crate) fn summary(&self) -> &str {
        &self.summary
    }

    pub(crate) fn font(&self) -> Option<FontPreset> {
        self.font
    }
}

pub(crate) fn split_header(source: &str) -> Result<(&str, &str), PageError> {
    let (opening, after_opening) = take_line(source);
    if opening.trim() != "---" {
        return Err(document_error(
            "Dosya '---' ile açılan bir başlık bloğuyla başlamalı.",
        ));
    }
    let mut remaining = after_opening;
    while !remaining.is_empty() {
        let (line, next) = take_line(remaining);
        if line.trim() == "---" {
            let header = &after_opening[..after_opening.len() - remaining.len()];
            return Ok((header, next));
        }
        remaining = next;
    }
    Err(document_error(
        "Başlık bloğunun kapanış satırı (---) bulunamadı.",
    ))
}

pub(crate) fn take_line(source: &str) -> (&str, &str) {
    let Some(newline) = source.find(['\r', '\n']) else {
        return (source, "");
    };
    let separator_bytes = if source[newline..].starts_with("\r\n") {
        2
    } else {
        1
    };
    source.split_at(newline + separator_bytes)
}

pub(crate) fn parse_fields(header: &str) -> Result<Mapping, PageError> {
    let mut value: Value = serde_yaml_ng::from_str(header).map_err(yaml_error)?;
    reject_custom_tags(&value)?;
    value.apply_merge().map_err(yaml_error)?;
    if is_falsey(&value) {
        return Ok(Mapping::new());
    }
    match value {
        Value::Mapping(fields) => Ok(fields),
        _ => Err(document_error(
            "Başlık bloğu 'anahtar: değer' satırlarından oluşmalı.",
        )),
    }
}

fn reject_custom_tags(value: &Value) -> Result<(), PageError> {
    match value {
        Value::Tagged(tagged) => Err(document_error(format!(
            "Başlık bloğu YAML olarak okunamadı: bilinmeyen etiket {}",
            tagged.tag
        ))),
        Value::Sequence(items) => items.iter().try_for_each(reject_custom_tags),
        Value::Mapping(fields) => fields.iter().try_for_each(|(key, value)| {
            reject_custom_tags(key)?;
            reject_custom_tags(value)
        }),
        _ => Ok(()),
    }
}

fn field_text(fields: &Mapping, key: &str) -> String {
    fields
        .get(key)
        .filter(|value| !is_falsey(value))
        .map(value_text)
        .unwrap_or_default()
}

fn parse_tags(value: Option<&Value>) -> Result<Vec<String>, PageError> {
    let Some(value) = value.filter(|value| !is_falsey(value)) else {
        return Ok(Vec::new());
    };
    match value {
        Value::String(tags) => Ok(tags
            .split(',')
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .map(str::to_owned)
            .collect()),
        Value::Sequence(tags) => Ok(tags.iter().map(value_text).collect()),
        _ => Err(document_error(
            "'etiketler:' alanı [a, b] biçiminde bir liste ya da 'a, b' biçiminde bir metin olmalı.",
        )),
    }
}

fn parse_font(value: Option<&Value>) -> Result<Option<FontPreset>, PageError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(name)) => name.parse().map(Some),
        Some(_) => Err(document_error("'font:' alanı bir yazı tipi adı olmalı.")),
    }
}

fn is_falsey(value: &Value) -> bool {
    match value {
        Value::Null | Value::Bool(false) => true,
        Value::Bool(true) => false,
        Value::Number(number) => number.as_f64() == Some(0.0),
        Value::String(text) => text.is_empty(),
        Value::Sequence(values) => values.is_empty(),
        Value::Mapping(fields) => fields.is_empty(),
        Value::Tagged(tagged) => is_falsey(&tagged.value),
    }
}

fn value_text(value: &Value) -> String {
    match value {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Sequence(items) => {
            let content = items.iter().map(value_repr).collect::<Vec<_>>().join(", ");
            format!("[{content}]")
        }
        Value::Mapping(fields) => {
            let content = fields
                .iter()
                .map(|(key, value)| format!("{}: {}", value_repr(key), value_repr(value)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{content}}}")
        }
        Value::Tagged(tagged) => value_text(&tagged.value),
    }
}

fn value_repr(value: &Value) -> String {
    if let Value::String(text) = value {
        return format!("'{}'", text.replace('\\', "\\\\").replace('\'', "\\'"));
    }
    value_text(value)
}

fn yaml_error(error: serde_yaml_ng::Error) -> PageError {
    document_error(format!("Başlık bloğu YAML olarak okunamadı: {error}"))
}

fn document_error(message: impl Into<String>) -> PageError {
    PageError::Document(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_keeps_original_body_with_crlf() {
        let source = "---\r\nbaslik: Başlık\r\n---\r\n# İlk 🧠\r\n";
        let (_, body) = Topic::parse(source).unwrap();
        assert_eq!(body, "# İlk 🧠\r\n");
    }

    #[test]
    fn parse_accepts_english_title() {
        let source = "---\ntitle: Project notes\n---\nDraft\n";
        let (topic, body) = Topic::parse(source).unwrap();
        assert_eq!(
            (topic.title(), topic.icon(), body),
            ("Project notes", "📄", "Draft\n")
        );
    }

    #[test]
    fn parse_prefers_the_existing_title_key() {
        let source = "---\ntitle: Project notes\nbaslik: Başlık\n---\n";
        let (topic, _) = Topic::parse(source).unwrap();
        assert_eq!(topic.title(), "Başlık");
    }

    #[test]
    fn parse_resolves_yaml_merge_before_applying_defaults() {
        let source = "---\ndefaults: &common {baslik: Başlık, ikon: 🧠}\n<<: *common\n---\n";
        let (topic, _) = Topic::parse(source).unwrap();
        assert_eq!((topic.title(), topic.icon()), ("Başlık", "🧠"));
    }

    #[test]
    fn parse_converts_numeric_tag_to_text() {
        let source = "---\nbaslik: Başlık\netiketler: [2026, llm]\n---\n";
        let (topic, _) = Topic::parse(source).unwrap();
        assert_eq!(topic.tags(), ["2026", "llm"]);
    }

    #[test]
    fn parse_rejects_unclosed_header() {
        let error = Topic::parse("---\nbaslik: Başlık\n").unwrap_err();
        assert!(error.to_string().contains("kapanış"));
    }

    #[test]
    fn parse_rejects_non_mapping_header() {
        let error = Topic::parse("---\n- Başlık\n---\n").unwrap_err();
        assert!(error.to_string().contains("anahtar: değer"));
    }

    #[test]
    fn parse_rejects_missing_title() {
        let error = Topic::parse("---\nikon: 🧠\n---\n").unwrap_err();
        assert!(error.to_string().contains("baslik"));
    }

    #[test]
    fn parse_rejects_custom_yaml_tags_even_in_unknown_fields() {
        let source = "---\nbaslik: Başlık\nmetadata: {item: !custom value}\n---\n";
        let error = Topic::parse(source).unwrap_err();
        assert!(error.to_string().contains("!custom"));
    }
}
