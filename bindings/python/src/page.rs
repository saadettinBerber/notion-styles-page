use pyo3::prelude::*;
use pyo3::types::PyTuple;
use serde::Serialize;

use crate::{frozen_field, value_error};

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
}

#[pymethods]
impl Topic {
    #[new]
    #[pyo3(signature = (title, body, icon = "📄", cover = "", date = "", tags = Vec::new(), summary = ""))]
    fn new(
        title: String,
        body: String,
        icon: &str,
        cover: &str,
        date: &str,
        tags: Vec<String>,
        summary: &str,
    ) -> Self {
        Self {
            title,
            body,
            icon: icon.into(),
            cover: cover.into(),
            date: date.into(),
            tags,
            summary: summary.into(),
        }
    }

    #[getter]
    fn tags<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.tags)
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
    #[pyo3(signature = (topic, body_html, *, lang = "tr", extra_styles = ""))]
    fn render(
        &self,
        topic: &Topic,
        body_html: &str,
        lang: &str,
        extra_styles: &str,
    ) -> PyResult<String> {
        let topic = serde_json::to_value(topic)
            .and_then(serde_json::from_value)
            .map_err(value_error)?;
        notion_sayfa::render_shell(&topic, body_html, lang, extra_styles).map_err(value_error)
    }
}

#[pyfunction]
pub(super) fn load_page_template() -> PageTemplate {
    PageTemplate
}
