use std::fmt::Write;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};

use base64::{Engine as _, engine::general_purpose::STANDARD};

const PREFIX: &str = "<!-- notion-sayfa-markdown:";
const VERSION: &str = "v1:";
const SUFFIX: &str = " -->";

pub(super) fn embed(html: &mut String, source: &str) {
    let checksum = crc32fast::hash(source.as_bytes());
    write!(html, "\n{PREFIX}{VERSION}{checksum:08x}:").expect("writing to a string cannot fail");
    // Standard base64 cannot contain the hyphens that close an HTML comment.
    STANDARD.encode_string(source, html);
    html.push_str(SUFFIX);
    html.push('\n');
}

pub(super) fn extract(html: &str) -> Result<Vec<u8>> {
    let footer = html
        .trim_ascii_end()
        .lines()
        .next_back()
        .and_then(|line| line.strip_prefix(PREFIX))
        .ok_or_else(|| invalid("HTML file has no embedded Markdown source"))?;
    let footer = footer.strip_suffix(SUFFIX).ok_or_else(corrupt)?;
    let footer = footer
        .strip_prefix(VERSION)
        .ok_or_else(|| invalid("unsupported embedded Markdown version"))?;
    let (checksum, encoded) = footer.split_once(':').ok_or_else(corrupt)?;
    let checksum = u32::from_str_radix(checksum, 16).map_err(|_| corrupt())?;
    let source = STANDARD.decode(encoded).map_err(|_| corrupt())?;
    if crc32fast::hash(&source) != checksum || str::from_utf8(&source).is_err() {
        return Err(corrupt());
    }
    Ok(source)
}

pub(super) fn has_embedded_source(file: &mut File) -> Result<bool> {
    let length = file.seek(SeekFrom::End(0))?;
    let start = last_nonblank_line_start(file, length)?;
    let mut prefix = [0; PREFIX.len()];
    if length - start < prefix.len() as u64 {
        return Ok(false);
    }
    file.seek(SeekFrom::Start(start))?;
    file.read_exact(&mut prefix)?;
    Ok(prefix == PREFIX.as_bytes())
}

fn last_nonblank_line_start(file: &mut File, mut end: u64) -> Result<u64> {
    let mut buffer = [0; 8192];
    let mut skipping_whitespace = true;
    while end > 0 {
        let start = end.saturating_sub(buffer.len() as u64);
        let chunk = &mut buffer[..(end - start) as usize];
        file.seek(SeekFrom::Start(start))?;
        file.read_exact(chunk)?;
        let chunk = if skipping_whitespace {
            let trimmed = chunk.trim_ascii_end();
            skipping_whitespace = trimmed.is_empty();
            trimmed
        } else {
            chunk
        };
        if let Some(newline) = chunk.iter().rposition(|&byte| byte == b'\n') {
            return Ok(start + newline as u64 + 1);
        }
        end = start;
    }
    Ok(0)
}

fn corrupt() -> Error {
    invalid("embedded Markdown source is damaged")
}

fn invalid(message: &'static str) -> Error {
    Error::new(ErrorKind::InvalidData, message)
}
