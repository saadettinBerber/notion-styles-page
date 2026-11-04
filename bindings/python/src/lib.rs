mod media;
mod page;
mod records;

use std::cell::RefCell;

use notion_sayfa::{
    DocumentFacts, MediaSource, PageError, PreparedDocument, ReaderOptions, RenderedBody,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyTuple};
use serde::Serialize;

#[pyclass(module = "notionmd_rust")]
struct MarkdownReader {
    media: Py<PyAny>,
    allow_raw_html: bool,
}

#[pymethods]
impl MarkdownReader {
    #[new]
    #[pyo3(signature = (media, *, allow_raw_html = true))]
    fn new(media: Py<PyAny>, allow_raw_html: bool) -> Self {
        Self {
            media,
            allow_raw_html,
        }
    }

    #[pyo3(signature = (source, *, id_prefix = ""))]
    fn prepare(&self, py: Python<'_>, source: &str, id_prefix: &str) -> PyResult<Prepared> {
        let document = notion_sayfa::prepare_document(
            source,
            ReaderOptions {
                id_prefix: id_prefix.into(),
                allow_raw_html: self.allow_raw_html,
            },
        )
        .map_err(value_error)?;
        Ok(Prepared {
            document,
            media: self.media.clone_ref(py),
        })
    }
}

#[pyclass(module = "notionmd_rust", name = "_Prepared", unsendable)]
struct Prepared {
    document: PreparedDocument,
    media: Py<PyAny>,
}

#[derive(Serialize)]
struct RenderedDocument<'a> {
    #[serde(flatten)]
    facts: &'a DocumentFacts,
    #[serde(flatten)]
    body: RenderedBody,
}

#[pymethods]
impl Prepared {
    fn document<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let view = records::decode(py, self.document.document())?;
        view.setattr("html", "")?;
        view.setattr("render_nodes", PyTuple::empty(py))?;
        Ok(view)
    }

    #[pyo3(signature = (media = None, *, embed_images = None))]
    fn render<'py>(
        &self,
        py: Python<'py>,
        media: Option<&Bound<'py, PyAny>>,
        embed_images: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let provider = PythonMedia {
            media: media.unwrap_or(self.media.bind(py)),
            embed_images,
            failure: RefCell::new(None),
        };
        let body = self.document.render(&provider);
        if let Some(error) = provider.failure.into_inner() {
            return Err(error);
        }
        records::decode(
            py,
            &RenderedDocument {
                facts: self.document.document(),
                body: body.map_err(value_error)?,
            },
        )
    }
}

struct PythonMedia<'a, 'py> {
    media: &'a Bound<'py, PyAny>,
    embed_images: Option<&'a Bound<'py, PyAny>>,
    failure: RefCell<Option<PyErr>>,
}

impl MediaSource for PythonMedia<'_, '_> {
    fn video_data_uri(&self, path: &str) -> Result<String, PageError> {
        self.method("video_data_uri", path)
    }

    fn image_data_uri(&self, path: &str) -> Result<String, PageError> {
        self.method("image_data_uri", path)
    }

    fn embedded_image(&self, path: &str) -> Result<Option<String>, PageError> {
        self.embed_images
            .map(|callback| self.extract(callback.call1((path,)), path))
            .transpose()
    }
}

impl PythonMedia<'_, '_> {
    fn method(&self, name: &str, path: &str) -> Result<String, PageError> {
        self.extract(self.media.call_method1(name, (path,)), path)
    }

    fn extract(&self, result: PyResult<Bound<'_, PyAny>>, path: &str) -> Result<String, PageError> {
        result.and_then(|value| value.extract()).map_err(|error| {
            self.failure.replace(Some(error));
            PageError::Media(format!("Medya okunamadı: {path}"))
        })
    }
}

#[pyfunction]
fn writing_guide() -> &'static str {
    notion_sayfa::writing_guide()
}

fn value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn frozen_field(py: Python<'_>, operation: &str, name: &str) -> PyResult<()> {
    let error = py.import("dataclasses")?.getattr("FrozenInstanceError")?;
    Err(PyErr::from_value(
        error.call1((format!("cannot {operation} field {name:?}"),))?,
    ))
}

#[pymodule]
fn notionmd_rust(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<MarkdownReader>()?;
    module.add_class::<media::MediaFolder>()?;
    module.add_class::<media::ImageInfo>()?;
    module.add_class::<page::Topic>()?;
    module.add_function(wrap_pyfunction!(media::inspect_image, module)?)?;
    module.add_function(wrap_pyfunction!(page::load_page_template, module)?)?;
    module.add_function(wrap_pyfunction!(writing_guide, module)?)
}
