use std::fs;
use std::path::Path;

use base64::{Engine, engine::general_purpose::STANDARD};

use crate::PageError;

mod folder;
mod raster;

pub use folder::MediaFolder;
pub use raster::{ImageInfo, inspect_image};

pub trait MediaSource {
    fn video_data_uri(&self, path: &str) -> Result<String, PageError>;
    fn image_data_uri(&self, path: &str) -> Result<String, PageError>;

    fn embedded_image(&self, _path: &str) -> Result<Option<String>, PageError> {
        Ok(None)
    }
}

/// Read video and poster files relative to the input document.
pub struct LocalMedia<'a> {
    root: &'a Path,
}

impl<'a> LocalMedia<'a> {
    pub fn new(root: &'a Path) -> Self {
        Self { root }
    }

    fn data_uri(&self, path: &str, kind: MediaKind) -> Result<String, PageError> {
        let media_type = kind.media_type(path)?;
        let content = fs::read(self.root.join(path))
            .map_err(|_| PageError::Media(format!("Medya okunamadı: {path}")))?;
        Ok(data_uri(media_type, &content))
    }
}

fn data_uri(media_type: &str, content: &[u8]) -> String {
    let encoded = STANDARD.encode(content);
    format!("data:{media_type};base64,{encoded}")
}

impl MediaSource for LocalMedia<'_> {
    fn video_data_uri(&self, path: &str) -> Result<String, PageError> {
        self.data_uri(path, MediaKind::Video)
    }

    fn image_data_uri(&self, path: &str) -> Result<String, PageError> {
        self.data_uri(path, MediaKind::Image)
    }
}

enum MediaKind {
    Video,
    Image,
}

impl MediaKind {
    fn media_type(&self, path: &str) -> Result<&'static str, PageError> {
        let (prefix, description) = match self {
            Self::Video => ("video/", "bir video dosyası (.mp4, .webm)"),
            Self::Image => ("image/", "bir görsel dosyası (.jpg, .png, .webp)"),
        };
        mime_guess::from_path(path)
            .first_raw()
            .filter(|media_type| media_type.starts_with(prefix))
            .ok_or_else(|| PageError::Media(format!("'{path}' {description} olmalı.")))
    }
}
