use std::path::PathBuf;

use pyo3::prelude::*;

use crate::{frozen_field, value_error};

#[pyclass(module = "notionmd_rust")]
pub(super) struct MediaFolder(notion_sayfa::MediaFolder);

#[pymethods]
impl MediaFolder {
    #[new]
    fn new(root: PathBuf) -> Self {
        Self(notion_sayfa::MediaFolder::new(root))
    }

    fn video_data_uri(&self, relative_path: &str) -> PyResult<String> {
        self.0.video_data_uri(relative_path).map_err(value_error)
    }

    fn image_data_uri(&self, relative_path: &str) -> PyResult<String> {
        self.0.image_data_uri(relative_path).map_err(value_error)
    }

    fn embedded_image_data_uri(&self, relative_path: &str) -> PyResult<String> {
        self.0
            .embedded_image_data_uri(relative_path)
            .map_err(value_error)
    }
}

#[pyclass(module = "notionmd_rust", frozen, get_all, eq, hash)]
#[derive(PartialEq, Eq, Hash)]
pub(super) struct ImageInfo {
    media_type: String,
    width: u32,
    height: u32,
}

#[pymethods]
impl ImageInfo {
    #[new]
    fn new(media_type: String, width: u32, height: u32) -> Self {
        Self {
            media_type,
            width,
            height,
        }
    }

    fn __setattr__(&self, py: Python<'_>, name: &str, _value: &Bound<'_, PyAny>) -> PyResult<()> {
        frozen_field(py, "assign to", name)
    }

    fn __delattr__(&self, py: Python<'_>, name: &str) -> PyResult<()> {
        frozen_field(py, "delete", name)
    }
}

#[pyfunction]
pub(super) fn inspect_image(content: &[u8]) -> PyResult<ImageInfo> {
    let info = notion_sayfa::inspect_image(content).map_err(value_error)?;
    Ok(ImageInfo::new(
        info.media_type.into(),
        info.width,
        info.height,
    ))
}
