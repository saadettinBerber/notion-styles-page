use pyo3::prelude::*;
use pyo3::types::PyTuple;
use serde::Serialize;

use crate::{frozen_field, value_error};
use notion_sayfa::{FontPreset, PageOptions};

#[pyclass(module = "notionmd_rust", frozen, eq, hash)]
#[derive(PartialEq, Eq, Hash, Serialize)]
pub(super) struct Topic {
    #[pyo3(get)]
    title: String,
    #[pyo3(get)]
    #[serde(skip)]
    body: String,
    #[pyo3(get)]
    icon: String,
    #[pyo3(get)]
    cover: String,
    #[pyo3(get)]
    date: String,
    tags: Vec<String>,
    #[pyo3(get)]
    summary: String,
    font: Option<FontPreset>,
}

#[pymethods]
impl Topic {
    #[new]
    #[pyo3(signature = (title, body, icon = "📄", cover = "", date = "", tags = Vec::new(), summary = "", font = None))]
    // Keep the existing positional Python constructor compatible.
    #[allow(clippy::too_many_arguments)]
    fn new(
        title: String,
        body: String,
        icon: &str,
        cover: &str,
        date: &str,
        tags: Vec<String>,
        summary: &str,
        font: Option<&str>,
    ) -> PyResult<Self> {
        Ok(Self {
            title,
            body,
            icon: icon.into(),
            cover: cover.into(),
            date: date.into(),
            tags,
            summary: summary.into(),
            font: parse_font(font)?,
        })
    }

    #[getter]
    fn tags<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.tags)
    }

    #[getter]
    fn font(&self) -> Option<String> {
        self.font.map(|font| font.to_string())
    }

    fn __setattr__(&self, py: Python<'_>, name: &str, _value: &Bound<'_, PyAny>) -> PyResult<()> {
        frozen_field(py, "assign to", name)
    }

    fn __delattr__(&self, py: Python<'_>, name: &str) -> PyResult<()> {
        frozen_field(py, "delete", name)
    }
}

#[pyclass(module = "notionmd_rust", name = "_PageTemplate")]
pub(super) struct PageTemplate;

#[pymethods]
impl PageTemplate {
    #[pyo3(signature = (topic, body_html, *, lang = "tr", extra_styles = "", font = None))]
    fn render(
        &self,
        topic: &Topic,
        body_html: &str,
        lang: &str,
        extra_styles: &str,
        font: Option<&str>,
    ) -> PyResult<String> {
        let topic = serde_json::to_value(topic)
            .and_then(serde_json::from_value)
            .map_err(value_error)?;
        let options = PageOptions {
            font: parse_font(font)?,
        };
        notion_sayfa::render_shell_with_options(&topic, body_html, lang, extra_styles, &options)
            .map_err(value_error)
    }
}

fn parse_font(font: Option<&str>) -> PyResult<Option<FontPreset>> {
    font.map(str::parse).transpose().map_err(value_error)
}

#[pyfunction]
pub(super) fn load_page_template() -> PageTemplate {
    PageTemplate
}
