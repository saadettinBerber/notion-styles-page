use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use serde_yaml_ng::Value;
use tempfile::NamedTempFile;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub(super) struct Plan {
    skills: Vec<SkillUpdate>,
}

struct SkillUpdate {
    name: String,
    destination: PathBuf,
    installations: BTreeSet<PathBuf>,
    linked: bool,
    files: Vec<FileUpdate>,
}

struct FileUpdate {
    relative: PathBuf,
    contents: Vec<u8>,
    previous: Option<Vec<u8>>,
    source_permissions: fs::Permissions,
}

pub(super) fn plan(source_skills: &Path, roots: &[PathBuf]) -> Result<Plan> {
    let source_skills = fs::canonicalize(source_skills)?;
    let mut source_files = BTreeMap::new();
    read_source_files(&source_skills, &mut source_files)?;
    let mut names = BTreeSet::new();
    let mut destinations: BTreeMap<PathBuf, (PathBuf, usize)> = BTreeMap::new();
    let mut skills: Vec<SkillUpdate> = Vec::new();

    for (document, contents) in &source_files {
        if document.file_name().is_none_or(|name| name != "SKILL.md") {
            continue;
        }
        let name = skill_name(document, contents)?;
        if !names.insert(name.clone()) {
            return Err(format!("duplicate source skill name {name}").into());
        }
        let source = document.parent().expect("source document has a parent");
        for root in roots {
            let installation = std::path::absolute(root.join(&name))?;
            let Some(destination) = installed_destination(&installation)? else {
                continue;
            };
            if let Some((previous_source, index)) = destinations.get(&destination) {
                if previous_source != source {
                    return Err(format!(
                        "{}: multiple source skills resolve to this destination",
                        destination.display()
                    )
                    .into());
                }
                skills[*index].installations.insert(installation);
                continue;
            }
            destinations.insert(destination.clone(), (source.to_path_buf(), skills.len()));
            let linked = destination == source;
            if !linked && destination.starts_with(&source_skills) {
                return Err(format!(
                    "{}: destination overlaps the source skills",
                    destination.display()
                )
                .into());
            }
            let mut files = Vec::new();
            if !linked {
                for (path, contents) in &source_files {
                    let Ok(relative) = path.strip_prefix(source) else {
                        continue;
                    };
                    let previous = read_destination_file(&destination, relative)?;
                    if previous.as_deref() != Some(contents.as_slice()) {
                        files.push(FileUpdate {
                            relative: relative.to_path_buf(),
                            contents: contents.clone(),
                            previous,
                            source_permissions: fs::metadata(path)?.permissions(),
                        });
                    }
                }
            }
            skills.push(SkillUpdate {
                name: name.clone(),
                destination,
                installations: BTreeSet::from([installation]),
                linked,
                files,
            });
        }
    }
    if names.is_empty() {
        return Err("No source skills found".into());
    }
    Ok(Plan { skills })
}

impl Plan {
    pub(super) fn apply(self) -> Result<()> {
        // Check every planned replacement before changing any destination file.
        for skill in &self.skills {
            for installation in &skill.installations {
                if fs::canonicalize(installation)? != skill.destination {
                    return Err(format!(
                        "{}: installed skill changed location during update",
                        installation.display()
                    )
                    .into());
                }
            }
            for file in &skill.files {
                if read_destination_file(&skill.destination, &file.relative)? != file.previous {
                    return Err(format!(
                        "{}: file changed during update",
                        skill.destination.join(&file.relative).display()
                    )
                    .into());
                }
            }
        }

        // Stage the complete write set before replacing any installed file.
        let mut staged = Vec::new();
        for skill in &self.skills {
            for file in &skill.files {
                let destination = skill.destination.join(&file.relative);
                let parent = destination.parent().expect("skill file has a parent");
                fs::create_dir_all(parent)?;
                let mut temporary = NamedTempFile::new_in(parent)?;
                temporary.write_all(&file.contents)?;
                let permissions = match &file.previous {
                    Some(_) => fs::metadata(&destination)?.permissions(),
                    None => file.source_permissions.clone(),
                };
                temporary.as_file().set_permissions(permissions)?;
                temporary.as_file().sync_all()?;
                staged.push((temporary, destination));
            }
        }
        for (temporary, destination) in staged {
            temporary
                .persist(&destination)
                .map_err(|error| format!("{}: {}", destination.display(), error.error))?;
        }

        if self.skills.is_empty() {
            println!("Skills: no installed skills found in the selected directories");
        }
        for skill in self.skills {
            let status = if skill.linked {
                "linked to source, up to date".to_owned()
            } else if skill.files.is_empty() {
                "up to date".to_owned()
            } else {
                format!("updated {} file(s)", skill.files.len())
            };
            println!(
                "Skill {}: {status} ({})",
                skill.name,
                skill.destination.display()
            );
        }
        Ok(())
    }
}

fn read_source_files(directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            read_source_files(&path, files)?;
        } else if kind.is_file() {
            files.insert(path.clone(), fs::read(&path)?);
        } else {
            return Err(format!(
                "{}: source must be a regular file or directory",
                path.display()
            )
            .into());
        }
    }
    Ok(())
}

fn skill_name(path: &Path, contents: &[u8]) -> Result<String> {
    let text = str::from_utf8(contents)?
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Err(format!("{}: missing skill frontmatter", path.display()).into());
    }
    let mut yaml = Vec::new();
    loop {
        match lines.next() {
            Some(line) if line.trim() == "---" => break,
            Some(line) => yaml.push(line),
            None => return Err(format!("{}: unclosed skill frontmatter", path.display()).into()),
        }
    }
    let mut metadata: Value = serde_yaml_ng::from_str(&yaml.join("\n"))?;
    metadata.apply_merge()?;
    let name = metadata
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{}: missing skill name", path.display()))?;
    if name.split('-').any(|part| {
        part.is_empty()
            || !part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    }) {
        return Err(format!("{}: invalid skill name", path.display()).into());
    }
    Ok(name.to_owned())
}

fn installed_destination(path: &Path) -> Result<Option<PathBuf>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_symlink() && !metadata.is_dir() {
        return Err(format!("{}: installed skill must be a directory", path.display()).into());
    }
    if !metadata.is_symlink() {
        match fs::symlink_metadata(path.join("SKILL.md")) {
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        }
    }
    let destination = fs::canonicalize(path)?;
    if !destination.is_dir() {
        return Err(format!("{}: installed skill must be a directory", path.display()).into());
    }
    Ok(Some(destination))
}

fn read_destination_file(directory: &Path, relative: &Path) -> Result<Option<Vec<u8>>> {
    let mut path = directory.to_path_buf();
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        path.push(component);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        if components.peek().is_some() {
            if !metadata.is_dir() {
                return Err(
                    format!("{}: expected a directory without symlinks", path.display()).into(),
                );
            }
        } else if !metadata.is_file() {
            return Err(format!(
                "{}: expected a regular file without symlinks",
                path.display()
            )
            .into());
        }
    }
    Ok(Some(fs::read(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{TempDir, tempdir};

    struct Fixture {
        _directory: TempDir,
        source: PathBuf,
        installed: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let directory = tempdir().unwrap();
            let source = directory.path().join("source");
            let installed = directory.path().join("installed");
            fs::create_dir_all(&source).unwrap();
            fs::create_dir_all(&installed).unwrap();
            Self {
                _directory: directory,
                source,
                installed,
            }
        }

        fn source_skill(&self, folder: &str, name: &str) -> PathBuf {
            let path = self.source.join(folder);
            write(
                &path.join("SKILL.md"),
                &format!("---\nname: {name}\n---\nNew guidance.\n"),
            );
            path
        }

        fn installed_skill(&self, name: &str) -> PathBuf {
            let path = self.installed.join(name);
            write(&path.join("SKILL.md"), "Old guidance.");
            path
        }

        fn plan(&self) -> Plan {
            plan(&self.source, std::slice::from_ref(&self.installed)).unwrap()
        }
    }

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn refreshes_installed_skills_and_missing_references_without_touching_extra_files() {
        let fixture = Fixture::new();
        let source = fixture.source_skill("nested/source-folder", "example");
        write(&source.join("references/guide.md"), "New reference.");
        fixture.source_skill("not-installed", "absent");
        fixture.source_skill("generate-notion-styles", "generate-notion-styles");
        let installed = fixture.installed_skill("example");
        let combined = fixture.installed_skill("generate-notion-styles");
        write(&installed.join("custom.md"), "Local notes.");

        fixture.plan().apply().unwrap();

        assert_eq!(
            fs::read(installed.join("SKILL.md")).unwrap(),
            fs::read(source.join("SKILL.md")).unwrap()
        );
        assert_eq!(
            fs::read_to_string(installed.join("references/guide.md")).unwrap(),
            "New reference."
        );
        assert_eq!(
            fs::read_to_string(installed.join("custom.md")).unwrap(),
            "Local notes."
        );
        assert!(
            fs::read_to_string(combined.join("SKILL.md"))
                .unwrap()
                .contains("New guidance.")
        );
        assert!(!fixture.installed.join("absent").exists());
    }

    #[test]
    fn second_update_leaves_current_files_in_place() {
        let fixture = Fixture::new();
        fixture.source_skill("example", "example");
        let installed = fixture.installed_skill("example").join("SKILL.md");
        fixture.plan().apply().unwrap();
        let before = fs::metadata(&installed).unwrap();

        let plan = fixture.plan();
        assert!(plan.skills[0].files.is_empty());
        plan.apply().unwrap();

        assert_eq!(
            before.modified().unwrap(),
            fs::metadata(installed).unwrap().modified().unwrap()
        );
    }

    #[test]
    fn destination_error_prevents_every_planned_write() {
        let fixture = Fixture::new();
        let source = fixture.source_skill("example", "example");
        write(&source.join("references/guide.md"), "New reference.");
        let installed = fixture.installed_skill("example");
        write(
            &installed.join("references"),
            "A file blocks the directory.",
        );

        assert!(plan(&fixture.source, std::slice::from_ref(&fixture.installed)).is_err());
        assert_eq!(
            fs::read_to_string(installed.join("SKILL.md")).unwrap(),
            "Old guidance."
        );
    }

    #[test]
    fn concurrent_destination_change_prevents_every_planned_write() {
        let fixture = Fixture::new();
        let source = fixture.source_skill("example", "example");
        write(&source.join("reference.md"), "New reference.");
        let installed = fixture.installed_skill("example");
        write(&installed.join("reference.md"), "Old reference.");
        let plan = fixture.plan();
        write(&installed.join("reference.md"), "Edited while updating.");

        assert!(plan.apply().is_err());
        assert_eq!(
            fs::read_to_string(installed.join("SKILL.md")).unwrap(),
            "Old guidance."
        );
        assert_eq!(
            fs::read_to_string(installed.join("reference.md")).unwrap(),
            "Edited while updating."
        );
    }

    #[cfg(unix)]
    #[test]
    fn source_links_are_already_current_and_remain_links() {
        use std::os::unix::fs::symlink;

        let fixture = Fixture::new();
        let source = fixture.source_skill("example", "example");
        let installed = fixture.installed.join("example");
        symlink(&source, &installed).unwrap();
        let plan = fixture.plan();

        assert!(plan.skills[0].linked);
        assert!(plan.skills[0].files.is_empty());
        plan.apply().unwrap();
        assert!(fs::symlink_metadata(installed).unwrap().is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn linked_agent_directories_update_the_canonical_copy_once() {
        use std::os::unix::fs::symlink;

        let fixture = Fixture::new();
        fixture.source_skill("example", "example");
        let canonical = fixture.installed_skill("example");
        let agent = fixture._directory.path().join("agent");
        fs::create_dir(&agent).unwrap();
        symlink(&canonical, agent.join("example")).unwrap();
        let plan = plan(&fixture.source, &[fixture.installed.clone(), agent.clone()]).unwrap();

        assert_eq!(plan.skills.len(), 1);
        assert_eq!(plan.skills[0].installations.len(), 2);
        plan.apply().unwrap();
        assert!(
            fs::read_to_string(agent.join("example/SKILL.md"))
                .unwrap()
                .contains("New guidance.")
        );
        assert!(
            fs::symlink_metadata(agent.join("example"))
                .unwrap()
                .is_symlink()
        );
    }

    #[cfg(unix)]
    #[test]
    fn retargeted_duplicate_alias_prevents_every_planned_write() {
        use std::os::unix::fs::symlink;

        let fixture = Fixture::new();
        fixture.source_skill("example", "example");
        let canonical = fixture.installed_skill("example");
        let agent = fixture._directory.path().join("agent");
        fs::create_dir(&agent).unwrap();
        let alias = agent.join("example");
        symlink(&canonical, &alias).unwrap();
        let plan = plan(&fixture.source, &[fixture.installed.clone(), agent]).unwrap();
        let replacement = fixture._directory.path().join("replacement");
        write(&replacement.join("SKILL.md"), "Replacement skill.");
        fs::remove_file(&alias).unwrap();
        symlink(&replacement, &alias).unwrap();

        let error = plan.apply().unwrap_err();

        assert!(error.to_string().contains("changed location during update"));
        assert_eq!(
            fs::read_to_string(canonical.join("SKILL.md")).unwrap(),
            "Old guidance."
        );
        assert_eq!(
            fs::read_to_string(alias.join("SKILL.md")).unwrap(),
            "Replacement skill."
        );
    }

    #[cfg(unix)]
    #[test]
    fn nested_destination_links_are_rejected_before_any_write() {
        use std::os::unix::fs::symlink;

        let fixture = Fixture::new();
        let source = fixture.source_skill("example", "example");
        write(&source.join("reference.md"), "New reference.");
        let installed = fixture.installed_skill("example");
        let outside = fixture._directory.path().join("outside.md");
        write(&outside, "Outside content.");
        symlink(&outside, installed.join("reference.md")).unwrap();

        assert!(plan(&fixture.source, std::slice::from_ref(&fixture.installed)).is_err());
        assert_eq!(fs::read_to_string(outside).unwrap(), "Outside content.");
        assert_eq!(
            fs::read_to_string(installed.join("SKILL.md")).unwrap(),
            "Old guidance."
        );
    }
}
