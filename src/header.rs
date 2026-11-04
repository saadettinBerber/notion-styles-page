use std::collections::HashMap;
use std::sync::LazyLock;

use libyaml_safer::{Event, EventData, Parser, ScalarStyle, Scanner, TokenData};
use serde::Serialize;

use crate::{PageError, Topic, topic};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SourceRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Serialize)]
pub struct HeaderField {
    pub name: String,
    pub value_range: SourceRange,
    pub text: Option<String>,
}

pub(crate) struct Header<'a> {
    pub topic: Option<Topic>,
    pub fields: Vec<HeaderField>,
    pub body: &'a str,
}

pub(crate) fn read(source: &str) -> Result<Header<'_>, PageError> {
    let body = source.strip_prefix('\u{feff}').unwrap_or(source);
    if topic::take_line(body).0.trim() != "---" {
        return Ok(Header {
            topic: None,
            fields: Vec::new(),
            body,
        });
    }
    let (yaml, body) = topic::split_header(body)?;
    let start = yaml.as_ptr() as usize - source.as_ptr() as usize;
    let fields = FieldReader::new(yaml, source[..start].chars().count())?.fields()?;
    let topic = Topic::from_fields(&topic::parse_fields(yaml)?)?;
    Ok(Header {
        topic: Some(topic),
        fields,
        body,
    })
}

fn yaml_error(error: impl std::fmt::Display) -> PageError {
    PageError::Document(format!("Başlık bloğu YAML olarak okunamadı: {error}"))
}

struct FieldReader<'a> {
    yaml: &'a str,
    offset: usize,
    parser: Parser<&'a [u8]>,
    scalar_starts: HashMap<usize, usize>,
    anchors: HashMap<String, Option<String>>,
}

struct FieldValue {
    text: Option<String>,
    start: usize,
    end: usize,
}

impl<'a> FieldReader<'a> {
    fn new(yaml: &'a str, offset: usize) -> Result<Self, PageError> {
        let scalar_starts = scalar_starts(yaml)?;
        let mut parser = Parser::new();
        parser.set_input(yaml.as_bytes());
        Ok(Self {
            yaml,
            offset,
            parser,
            scalar_starts,
            anchors: HashMap::new(),
        })
    }

    fn event(&mut self) -> Result<Event, PageError> {
        self.parser.parse().map_err(yaml_error)
    }

    fn fields(mut self) -> Result<Vec<HeaderField>, PageError> {
        self.event()?;
        if matches!(self.event()?.data, EventData::StreamEnd) {
            return Ok(Vec::new());
        }
        let root = self.event()?;
        if !matches!(root.data, EventData::MappingStart { .. }) {
            return Err(yaml_error("anahtar: değer satırları bekleniyor"));
        }
        self.mapping_fields()
    }

    fn mapping_fields(&mut self) -> Result<Vec<HeaderField>, PageError> {
        let mut fields = Vec::new();
        loop {
            let key = self.event()?;
            if matches!(key.data, EventData::MappingEnd) {
                return Ok(fields);
            }
            let name = match &key.data {
                EventData::Scalar { value, .. } => Some(value.clone()),
                _ => None,
            };
            self.consume(key)?;
            let value = self.event()?;
            let value = self.consume(value)?;
            if let Some(name) = name {
                fields.push(self.field(name, value)?);
            }
        }
    }

    fn field(&self, name: String, value: FieldValue) -> Result<HeaderField, PageError> {
        let point = |byte| {
            self.yaml
                .get(..byte)
                .map(|s| self.offset + s.chars().count())
        };
        let start = point(value.start).ok_or_else(|| yaml_error("geçersiz başlangıç konumu"))?;
        let end = point(value.end).ok_or_else(|| yaml_error("geçersiz bitiş konumu"))?;
        Ok(HeaderField {
            name,
            value_range: SourceRange { start, end },
            text: value.text,
        })
    }

    fn consume(&mut self, event: Event) -> Result<FieldValue, PageError> {
        let start = event.start_mark.index as usize;
        let end = event.end_mark.index as usize;
        match event.data {
            EventData::Scalar {
                anchor,
                tag,
                value,
                style,
                ..
            } => Ok(self.scalar(anchor, tag.as_deref(), value, style, start..end)),
            EventData::Alias { anchor } => Ok(FieldValue {
                text: self.anchors.get(&anchor).cloned().flatten(),
                start,
                end,
            }),
            EventData::MappingStart { .. } | EventData::SequenceStart { .. } => Ok(FieldValue {
                text: None,
                start,
                end: self.compound_end()?,
            }),
            _ => Err(yaml_error("beklenmeyen YAML değeri")),
        }
    }

    fn compound_end(&mut self) -> Result<usize, PageError> {
        let mut depth = 1;
        loop {
            let event = self.event()?;
            match event.data {
                EventData::MappingStart { .. } | EventData::SequenceStart { .. } => depth += 1,
                EventData::MappingEnd | EventData::SequenceEnd => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(event.end_mark.index as usize);
                    }
                }
                EventData::Scalar {
                    anchor: Some(anchor),
                    tag,
                    value,
                    style,
                    ..
                } => {
                    self.anchors
                        .insert(anchor, string_value(tag.as_deref(), value, style));
                }
                EventData::StreamEnd => return Err(yaml_error("eksik YAML değeri")),
                _ => {}
            }
        }
    }

    fn scalar(
        &mut self,
        anchor: Option<String>,
        tag: Option<&str>,
        value: String,
        style: ScalarStyle,
        span: std::ops::Range<usize>,
    ) -> FieldValue {
        let start = self
            .scalar_starts
            .get(&span.end)
            .copied()
            .filter(|start| *start >= span.start)
            .unwrap_or(span.start);
        let end = if matches!(style, ScalarStyle::Literal | ScalarStyle::Folded) {
            self.yaml.get(start..span.end).map_or(span.end, |raw| {
                start + raw.trim_end_matches(['\r', '\n']).len()
            })
        } else {
            span.end
        };
        let text = string_value(tag, value, style);
        if let Some(anchor) = anchor {
            self.anchors.insert(anchor, text.clone());
        }
        FieldValue { text, start, end }
    }
}

fn scalar_starts(yaml: &str) -> Result<HashMap<usize, usize>, PageError> {
    let mut scanner = Scanner::new();
    scanner.set_input(yaml.as_bytes());
    let mut starts = HashMap::new();
    loop {
        let token = Scanner::scan(&mut scanner).map_err(yaml_error)?;
        match token.data {
            TokenData::StreamEnd => return Ok(starts),
            TokenData::Scalar { .. } => {
                starts.insert(
                    token.end_mark.index as usize,
                    token.start_mark.index as usize,
                );
            }
            _ => {}
        }
    }
}

fn string_value(tag: Option<&str>, value: String, style: ScalarStyle) -> Option<String> {
    if value.is_empty() {
        return None;
    }
    let is_string = match tag {
        Some(tag) => tag == "tag:yaml.org,2002:str" || tag == "!",
        None if style != ScalarStyle::Plain => true,
        None => !implicit_non_string(&value),
    };
    is_string.then_some(value)
}

fn implicit_non_string(value: &str) -> bool {
    static DATE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^\d{4}-\d{1,2}-\d{1,2}(?:$|[Tt ][ \t]*\d{1,2}:\d{2}:\d{2})").unwrap()
    });
    matches!(
        value.to_ascii_lowercase().as_str(),
        "yes" | "no" | "on" | "off" | "<<"
    ) || DATE.is_match(value)
        || matches!(
            serde_yaml_ng::from_str::<serde_yaml_ng::Value>(value),
            Ok(serde_yaml_ng::Value::Bool(_)
                | serde_yaml_ng::Value::Null
                | serde_yaml_ng::Value::Number(_))
        )
}
