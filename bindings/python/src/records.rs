use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyTuple};
use serde::Serialize;

use crate::value_error;

pub(super) fn decode<'py>(py: Python<'py>, source: &impl Serialize) -> PyResult<Bound<'py, PyAny>> {
    let json = serde_json::to_string(source).map_err(value_error)?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("object_hook", wrap_pyfunction!(record, py)?)?;
    py.import("json")?
        .getattr("loads")?
        .call((json,), Some(&kwargs))
}

#[pyfunction]
fn record<'py>(py: Python<'py>, fields: &Bound<'py, PyDict>) -> PyResult<Bound<'py, PyAny>> {
    if let Some(bytes) = fields.get_item("$bytes")? {
        return Ok(python_bytes(py, &bytes)?.into_any());
    }
    if let Some(bytes) = fields.get_item("content_bytes")? {
        fields.set_item("content_bytes", python_bytes(py, &bytes)?)?;
    }
    if let Some(shape) = fields.get_item("shape")? {
        fields.set_item("shape", tuple_shape(py, shape)?)?;
    }
    py.import("types")?
        .getattr("SimpleNamespace")?
        .call((), Some(fields))
}

fn python_bytes<'py>(py: Python<'py>, value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyBytes>> {
    Ok(PyBytes::new(py, &value.extract::<Vec<u8>>()?))
}

fn tuple_shape<'py>(py: Python<'py>, value: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    let Ok(items) = value.cast::<PyList>() else {
        return Ok(value);
    };
    let shape = items
        .iter()
        .map(|item| tuple_shape(py, item))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, shape)?.into_any())
}
