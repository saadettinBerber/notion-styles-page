//! Compose the installable skill from discovered skill metadata and bodies.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::sync::LazyLock;

use clap::Parser;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use regex::Regex;
use serde_yaml_ng::{Mapping, Value};

const OUTPUT: &str = "skill/generate-notion-styles/SKILL.md";
const TEMPLATE: &str = "scripts/generate-skill.template.md";
const SOURCE_URL: &str = "https://github.com/saadettinBerber/notion-styles-page/blob/main/";
const PATH_ENCODING: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'/')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

static NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap());
static FENCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s{0,3}(`{3,}|~{3,})(.*)$").unwrap());
static INLINE_CODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`+[^`]*`+").unwrap());
static INLINE_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\]\()([^\s()]+)(\))").unwrap());
static REFERENCE_LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(^\s{0,3}\[[^]]+\]:\s*)(\S+)(.*$)").unwrap());
static SCHEME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9+.-]*:").unwrap());
static HEADING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(#{1,4}) ").unwrap());

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(about = "Compose the installable skill from discovered skill metadata and bodies.")]
struct Args {
    /// Fail if the generated file is missing or stale; do not write.
    #[arg(long)]
    check: bool,
}

fn main() -> ExitCode {
    match update(Path::new(env!("CARGO_MANIFEST_DIR")), Args::parse().check) {
        Ok(true) => println!("Generated {OUTPUT}"),
        Ok(false) => println!("{OUTPUT} is up to date"),
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

fn read_text(path: &Path) -> std::io::Result<String> {
    fs::read_to_string(path).map(|text| text.replace("\r\n", "\n").replace('\r', "\n"))
}

fn read_document(path: &Path, required: bool) -> Result<(Mapping, String)> {
    let text = read_text(path)?;
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        if required {
            return Err(format!("{}: missing YAML frontmatter", path.display()).into());
        }
        return Ok((Mapping::new(), text.trim().to_owned()));
    }
    let mut yaml = Vec::new();
    loop {
        match lines.next() {
            Some(line) if line.trim() == "---" => break,
            Some(line) => yaml.push(line),
            None => return Err(format!("{}: unclosed YAML frontmatter", path.display()).into()),
        }
    }
    let mut data: Value = serde_yaml_ng::from_str(&yaml.join("\n"))?;
    data.apply_merge()?;
    let Value::Mapping(data) = data else {
        return Err(format!("{}: frontmatter must be a mapping", path.display()).into());
    };
    if required {
        let name = required_string(&data, "name", path)?;
        required_string(&data, "description", path)?;
        if !NAME.is_match(name) {
            return Err(format!("{}: invalid skill name", path.display()).into());
        }
    }
    Ok((data, lines.collect::<Vec<_>>().join("\n").trim().to_owned()))
}

fn required_string<'a>(data: &'a Mapping, key: &str, path: &Path) -> Result<&'a str> {
    data.get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| format!("{}: {key} must be a nonempty string", path.display()).into())
}

// Canonicalize existing ancestors while allowing links to documents not yet written.
fn contained(path: &Path, root: &Path) -> Result<PathBuf> {
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir => resolved.push(component.as_os_str()),
            other => {
                resolved.push(other.as_os_str());
                match fs::symlink_metadata(&resolved) {
                    Ok(_) => resolved = fs::canonicalize(&resolved)?,
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
        }
    }
    if !resolved.starts_with(root) {
        return Err(format!(
            "{}: source must stay inside {}",
            path.display(),
            root.display()
        )
        .into());
    }
    Ok(resolved)
}

fn repository_path(path: &Path, root: &Path) -> Result<String> {
    let relative = path.strip_prefix(root)?;
    Ok(relative
        .to_str()
        .ok_or("source path must be valid UTF-8")?
        .replace(std::path::MAIN_SEPARATOR, "/"))
}

fn destination(target: &str, source: &Path, root: &Path) -> Result<String> {
    if SCHEME.is_match(target) || target.starts_with("//") || target.starts_with('#') {
        return Ok(target.to_owned());
    }
    let (target, fragment) = target.split_once('#').unwrap_or((target, ""));
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let path = contained(&source.parent().unwrap().join(path), root)?;
    let relative = repository_path(&path, root)?;
    let mut url = format!(
        "{SOURCE_URL}{}",
        utf8_percent_encode(&relative, PATH_ENCODING)
    );
    if !query.is_empty() {
        url.push('?');
        url.push_str(query);
    }
    if !fragment.is_empty() {
        url.push('#');
        url.push_str(fragment);
    }
    Ok(url)
}

fn rewrite_links(text: &str, pattern: &Regex, source: &Path, root: &Path) -> Result<String> {
    let mut output = String::new();
    let mut end = 0;
    for captures in pattern.captures_iter(text) {
        let matched = captures.get(0).unwrap();
        output.push_str(&text[end..matched.start()]);
        output.push_str(&captures[1]);
        output.push_str(&destination(&captures[2], source, root)?);
        output.push_str(&captures[3]);
        end = matched.end();
    }
    output.push_str(&text[end..]);
    Ok(output)
}

fn portable_body(body: &str, source: &Path, root: &Path) -> Result<String> {
    let mut output = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    for line in body.lines() {
        let marker = FENCE.captures(line);
        if let Some((kind, length)) = fence {
            if let Some(marker) = &marker
                && marker[1].as_bytes()[0] == kind
                && marker[1].len() >= length
                && marker[2].trim().is_empty()
            {
                fence = None;
            }
            output.push(line.to_owned());
            continue;
        }
        if let Some(marker) = marker {
            fence = Some((marker[1].as_bytes()[0], marker[1].len()));
            output.push(line.to_owned());
            continue;
        }
        let mut converted = String::new();
        let mut end = 0;
        for code in INLINE_CODE.find_iter(line) {
            converted.push_str(&rewrite_links(
                &rewrite_links(&line[end..code.start()], &INLINE_LINK, source, root)?,
                &REFERENCE_LINK,
                source,
                root,
            )?);
            converted.push_str(code.as_str());
            end = code.end();
        }
        converted.push_str(&rewrite_links(
            &rewrite_links(&line[end..], &INLINE_LINK, source, root)?,
            &REFERENCE_LINK,
            source,
            root,
        )?);
        output.push(HEADING.replace(&converted, "##${1} ").into_owned());
    }
    Ok(output.join("\n"))
}

fn discover(directory: &Path, output: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.starts_with(output) {
            continue;
        }
        if entry.file_type()?.is_dir() {
            discover(&path, output, paths)?;
        } else if entry.file_name() == "SKILL.md" {
            paths.push(path);
        }
    }
    Ok(())
}

fn generate(root: &Path) -> Result<String> {
    let root = fs::canonicalize(root)?;
    let skill_root = root.join("skill");
    let output = root.join(OUTPUT);
    let (template_data, _) = read_document(&root.join(TEMPLATE), true)?;
    let mut names =
        BTreeSet::from([required_string(&template_data, "name", Path::new(TEMPLATE))?.to_owned()]);
    let mut paths = Vec::new();
    if skill_root.exists() {
        discover(&skill_root, output.parent().unwrap(), &mut paths)?;
    }
    paths.sort();
    let mut sections = BTreeMap::new();
    for path in paths {
        contained(&path, &skill_root)?;
        let (data, mut body) = read_document(&path, true)?;
        let name = required_string(&data, "name", &path)?;
        if !names.insert(name.to_owned()) {
            return Err(format!("{}: duplicate skill name {name}", path.display()).into());
        }
        let mut source = path.clone();
        if let Some(metadata) = data.get("metadata") {
            let metadata = metadata
                .as_mapping()
                .ok_or_else(|| format!("{}: metadata must be a mapping", path.display()))?;
            if metadata.contains_key("source") {
                let relative = required_string(metadata, "source", &path)?;
                source = contained(&path.parent().unwrap().join(relative), &skill_root)?;
                if source.starts_with(output.parent().unwrap())
                    || source == fs::canonicalize(&path)?
                {
                    return Err(format!(
                        "{}: metadata.source cannot point to itself or the generated skill",
                        path.display()
                    )
                    .into());
                }
                let (source_data, source_body) = read_document(&source, false)?;
                if let Some(source_name) = source_data.get("name")
                    && source_name.as_str() != Some(name)
                {
                    return Err(format!(
                        "{}: source skill name does not match {name}",
                        path.display()
                    )
                    .into());
                }
                body = source_body;
            }
        }
        if body.is_empty() {
            return Err(format!("{}: empty skill body", source.display()).into());
        }
        let description = required_string(&data, "description", &path)?
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let relative = repository_path(&source, &root)?;
        let encoded = utf8_percent_encode(&relative, PATH_ENCODING);
        sections.insert(
            name.to_owned(),
            format!(
                "## {name}\n\n{description}\n\nSource: [{relative}]({SOURCE_URL}{encoded})\n\n{}",
                portable_body(&body, &source, &root)?
            ),
        );
    }
    if sections.is_empty() {
        return Err("No source skills found".into());
    }
    Ok(format!(
        "{}\n\n{}\n",
        read_text(&root.join(TEMPLATE))?.trim_end(),
        sections
            .into_values()
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    ))
}

fn update(root: &Path, check: bool) -> Result<bool> {
    let content = generate(root)?;
    let output = root.join(OUTPUT);
    let current = match read_text(&output) {
        Ok(current) => Some(current),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    if current.as_deref() == Some(&content) {
        return Ok(false);
    }
    if check {
        return Err(
            format!("{OUTPUT} is stale; run cargo run --locked --example generate-skill").into(),
        );
    }
    fs::create_dir_all(output.parent().unwrap())?;
    fs::write(output, content)?;
    Ok(true)
}

#[cfg(test)]
#[path = "generate_skill_tests.rs"]
mod tests;
