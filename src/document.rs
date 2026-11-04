use serde::Serialize;

use crate::{HeaderField, MediaSource, PageError, RenderedBody, Topic, header, markdown};

#[derive(Debug, Default)]
pub struct ReaderOptions {
    pub id_prefix: String,
    pub allow_raw_html: bool,
}

#[derive(Serialize)]
pub struct DocumentFacts {
    pub source: String,
    pub topic: Option<Topic>,
    pub header_fields: Vec<HeaderField>,
    #[serde(flatten)]
    pub analysis: markdown::Analysis,
}

pub struct PreparedDocument {
    facts: DocumentFacts,
    markdown: markdown::PreparedMarkdown,
    options: ReaderOptions,
}

pub fn prepare_document(
    source: &str,
    options: ReaderOptions,
) -> Result<PreparedDocument, PageError> {
    let header = header::read(source)?;
    let offset = source.len() - header.body.len();
    let markdown = markdown::prepare_reader(header.body);
    let analysis = markdown::analyze(&markdown, source, offset)?;
    let facts = DocumentFacts {
        source: source.to_owned(),
        topic: header.topic,
        header_fields: header.fields,
        analysis,
    };
    Ok(PreparedDocument {
        facts,
        markdown,
        options,
    })
}

impl PreparedDocument {
    pub fn document(&self) -> &DocumentFacts {
        &self.facts
    }

    pub fn render(&self, media: &impl MediaSource) -> Result<RenderedBody, PageError> {
        markdown::render_document(&self.markdown, media, &self.options)
    }
}
