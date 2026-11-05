use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{self, Seek, Write};
use std::path::{Component, Path, PathBuf};

use notion_sayfa::{LocalMedia, PageError, PageOptions, prepare_page, render_page_with_options};
use tempfile::NamedTempFile;

use super::archive;

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Convert,
    Archive,
    Restore,
}

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
    mode: Mode,
    options: &PageOptions,
    mut report: impl FnMut(&Path, Result<PathBuf, BuildError>),
) -> Result<bool, BuildError> {
    let mut succeeded = true;
    for batch in group_sources(sources) {
        let destination = output_dir.unwrap_or(&batch.directory);
        let media = LocalMedia::new(&batch.directory);
        fs::create_dir_all(destination).map_err(|error| file_error(destination, error))?;
        for source in batch.sources {
            let result = match mode {
                Mode::Convert => build_page(source, destination, &media, options, false),
                Mode::Archive => build_page(source, destination, &media, options, true),
                Mode::Restore => restore(source, destination),
            };
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
    options: &PageOptions,
    archive_source: bool,
) -> Result<PathBuf, BuildError> {
    if archive_source {
        validate_archive_source(source)?;
    }
    let text = fs::read_to_string(source).map_err(|error| file_error(source, error))?;
    let page = prepare_page(&text).map_err(BuildError::Page)?;
    let mut html = render_page_with_options(&page, media, options).map_err(BuildError::Page)?;
    let output = output_path(source, destination, ".html");
    if archive_source {
        archive::embed(&mut html, &text);
        write_new_file(&output, html.as_bytes()).map_err(|error| file_error(&output, error))?;
        remove_archived_source(source, &text)?;
    } else {
        write_page(&output, html.as_bytes()).map_err(|error| file_error(&output, error))?;
    }
    Ok(output)
}

fn validate_archive_source(source: &Path) -> Result<(), BuildError> {
    let metadata = fs::symlink_metadata(source).map_err(|error| file_error(source, error))?;
    if !metadata.is_file()
        || !source
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        return Err(file_error(
            source,
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "archive requires a regular .md file",
            ),
        ));
    }
    Ok(())
}

fn remove_archived_source(source: &Path, archived: &str) -> Result<(), BuildError> {
    validate_archive_source(source)?;
    let current = fs::read(source).map_err(|error| file_error(source, error))?;
    if current != archived.as_bytes() {
        return Err(file_error(
            source,
            io::Error::other("Markdown changed during conversion and was kept"),
        ));
    }
    fs::remove_file(source).map_err(|error| file_error(source, error))
}

fn restore(source: &Path, destination: &Path) -> Result<PathBuf, BuildError> {
    let html = fs::read_to_string(source).map_err(|error| file_error(source, error))?;
    let markdown = archive::extract(&html).map_err(|error| file_error(source, error))?;
    let output = output_path(source, destination, ".md");
    write_new_file(&output, &markdown).map_err(|error| file_error(&output, error))?;
    Ok(output)
}

fn write_page(output: &Path, contents: &[u8]) -> io::Result<()> {
    // Inspect the same open file we will write so an archive cannot appear between the check and open.
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(output)?;
    if archive::has_embedded_source(&mut file)? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "output contains an embedded Markdown source and was kept",
        ));
    }
    file.rewind()?;
    file.set_len(0)?;
    file.write_all(contents)
}

fn write_new_file(output: &Path, contents: &[u8]) -> io::Result<()> {
    let directory = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = NamedTempFile::new_in(directory)?;
    file.write_all(contents)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(output)
        .map_err(|error| error.error)?;
    #[cfg(unix)]
    fs::File::open(directory)?.sync_all()?;
    Ok(())
}

fn output_path(source: &Path, destination: &Path, suffix: &str) -> PathBuf {
    let mut filename = source.file_stem().unwrap_or_default().to_os_string();
    filename.push(suffix);
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
