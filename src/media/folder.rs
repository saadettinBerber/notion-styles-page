use std::io;
use std::path::{Component, Path, PathBuf};

use percent_encoding::percent_decode_str;

use super::{LocalMedia, MediaSource, data_uri, inspect_image};
use crate::PageError;

/// Resolve media paths relative to a document directory.
pub struct MediaFolder {
    root: PathBuf,
}

impl MediaFolder {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn video_data_uri(&self, path: &str) -> Result<String, PageError> {
        LocalMedia::new(&self.root).video_data_uri(path)
    }

    pub fn image_data_uri(&self, path: &str) -> Result<String, PageError> {
        LocalMedia::new(&self.root).image_data_uri(path)
    }

    /// Open and validate a local raster before embedding its original bytes.
    pub fn embedded_image_data_uri(&self, target: &str) -> Result<String, PageError> {
        let error = |reason: &dyn std::fmt::Display| {
            PageError::Media(format!("Görsel gömülemedi: {target}: {reason}"))
        };
        let path = image_path(target).map_err(|reason| error(&reason))?;
        let content = read_regular_file(&self.root, &path).map_err(|reason| error(&reason))?;
        let image = inspect_image(&content).map_err(|reason| error(&reason))?;
        Ok(data_uri(image.media_type, &content))
    }
}

impl MediaSource for MediaFolder {
    fn video_data_uri(&self, path: &str) -> Result<String, PageError> {
        self.video_data_uri(path)
    }

    fn image_data_uri(&self, path: &str) -> Result<String, PageError> {
        self.image_data_uri(path)
    }

    fn embedded_image(&self, path: &str) -> Result<Option<String>, PageError> {
        self.embedded_image_data_uri(path).map(Some)
    }
}

fn image_path(target: &str) -> Result<PathBuf, &'static str> {
    if target
        .as_bytes()
        .windows(3)
        .any(|chunk| chunk.eq_ignore_ascii_case(b"%2f") || chunk.eq_ignore_ascii_case(b"%5c"))
    {
        return Err("Görsel hedefinde kodlanmış yol ayırıcı olamaz.");
    }
    let decoded = percent_decode_str(target)
        .decode_utf8()
        .map_err(|_| "Görsel yolu UTF-8 olmalı.")?;
    let path = Path::new(decoded.as_ref());
    if path.is_absolute()
        || path.file_name().is_none()
        || path.components().any(|part| part == Component::ParentDir)
        || decoded.contains('\\')
    {
        return Err("Görsel yolu belge klasörü içinde olmalı.");
    }
    Ok(path.to_owned())
}

#[cfg(unix)]
fn read_regular_file(root: &Path, path: &Path) -> io::Result<Vec<u8>> {
    use std::fs::File;
    use std::io::Read;

    use rustix::fs::{Mode, OFlags, open, openat};

    let directory_flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC;
    let mut directory = open(root, directory_flags, Mode::empty())?;
    let mut components = path.components();
    let filename = components
        .next_back()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Görsel yolu boş."))?;
    for component in components {
        directory = openat(
            &directory,
            component.as_os_str(),
            directory_flags | OFlags::NOFOLLOW,
            Mode::empty(),
        )?;
    }
    let file_flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let descriptor = openat(&directory, filename.as_os_str(), file_flags, Mode::empty())?;
    let mut file = File::from(descriptor);
    if !file.metadata()?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Görsel düzenli dosya olmalı.",
        ));
    }
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

#[cfg(not(unix))]
fn read_regular_file(_root: &Path, _path: &Path) -> io::Result<Vec<u8>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Yerel görsel gömme Unix dosya erişimi gerektirir.",
    ))
}
