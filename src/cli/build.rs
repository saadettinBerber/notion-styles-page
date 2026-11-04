use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use notion_sayfa::{LocalMedia, PageError, prepare_page, render_page};

struct Batch<'a> {
    directory: PathBuf,
    sources: Vec<&'a Path>,
}

#[derive(Debug)]
pub(crate) enum BuildError {
    File {
        path: PathBuf,
        error: std::io::Error,
    },
    Page(PageError),
}

impl fmt::Display for BuildError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File { path, error } => write!(output, "{}: {error}", path.display()),
            Self::Page(error) => error.fmt(output),
        }
    }
}

pub(crate) fn run(
    sources: &[PathBuf],
    output_dir: Option<&Path>,
    mut report: impl FnMut(&Path, Result<PathBuf, BuildError>),
) -> Result<bool, BuildError> {
    let mut succeeded = true;
    for batch in group_sources(sources) {
        let destination = output_dir.unwrap_or(&batch.directory);
        let media = LocalMedia::new(&batch.directory);
        fs::create_dir_all(destination).map_err(|error| file_error(destination, error))?;
        for source in batch.sources {
            let result = build_page(source, destination, &media);
            succeeded &= result.is_ok();
            report(source, result);
        }
    }
    Ok(succeeded)
}

fn group_sources(sources: &[PathBuf]) -> Vec<Batch<'_>> {
    let mut indexes = HashMap::new();
    let mut batches = Vec::new();
    for source in sources {
        let directory = source_directory(source);
        let index = *indexes.entry(directory.clone()).or_insert_with(|| {
            batches.push(Batch {
                directory,
                sources: Vec::new(),
            });
            batches.len() - 1
        });
        batches[index].sources.push(source);
    }
    batches
}

fn source_directory(source: &Path) -> PathBuf {
    source
        .parent()
        .unwrap_or(Path::new(""))
        .components()
        .filter(|part| *part != Component::CurDir)
        .collect()
}

fn build_page(
    source: &Path,
    destination: &Path,
    media: &LocalMedia<'_>,
) -> Result<PathBuf, BuildError> {
    let text = fs::read_to_string(source).map_err(|error| file_error(source, error))?;
    let page = prepare_page(&text).map_err(BuildError::Page)?;
    let html = render_page(&page, media).map_err(BuildError::Page)?;
    let output = output_path(source, destination);
    fs::write(&output, html).map_err(|error| file_error(&output, error))?;
    Ok(output)
}

fn output_path(source: &Path, destination: &Path) -> PathBuf {
    let mut filename = source.file_stem().unwrap_or_default().to_os_string();
    filename.push(".html");
    destination.join(filename)
}

fn file_error(path: &Path, error: std::io::Error) -> BuildError {
    BuildError::File {
        path: path.to_owned(),
        error,
    }
}

pub(crate) fn display_name(path: &Path) -> std::borrow::Cow<'_, str> {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
}
