use super::*;
use tempfile::{TempDir, tempdir};

struct Fixture {
    _directory: TempDir,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempdir().unwrap();
        let root = fs::canonicalize(directory.path()).unwrap();
        let fixture = Self {
            _directory: directory,
            root,
        };
        fixture.write(TEMPLATE, include_str!("generate-skill.template.md"));
        fixture
    }

    fn write(&self, relative: impl AsRef<Path>, text: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        path
    }

    fn skill(&self, folder: &str, name: &str, body: &str) -> PathBuf {
        self.write(
            format!("skill/{folder}/SKILL.md"),
            &format!("---\nname: {name}\ndescription: Use for examples.\n---\n\n{body}\n"),
        )
    }

    fn source_skill(&self, source: &str) -> PathBuf {
        self.write(
            "skill/example/SKILL.md",
            &format!("---\nname: example\ndescription: Use for examples.\nmetadata:\n  source: {source}\n---\nRead the guide through the CLI.\n"),
        )
    }
}

#[test]
fn discovers_added_updated_and_deleted_skills_without_configuration() {
    let fixture = Fixture::new();
    fixture.skill("example", "example", "Keep the context.");
    let before = generate(&fixture.root).unwrap();
    let path = fixture.skill("nested/new-skill", "new-skill", "New guidance.");
    assert!(generate(&fixture.root).unwrap().contains("New guidance."));
    fixture.skill("nested/new-skill", "new-skill", "Changed guidance.");
    let changed = generate(&fixture.root).unwrap();
    assert!(changed.contains("Changed guidance."));
    assert!(!changed.contains("New guidance."));
    fs::remove_file(path).unwrap();
    assert_eq!(before, generate(&fixture.root).unwrap());
}

#[test]
fn excludes_generated_subtree_and_does_not_rewrite_identical_output() {
    let fixture = Fixture::new();
    fixture.skill("example", "example", "Keep the context.");
    assert!(update(&fixture.root, false).unwrap());
    let output = fixture.root.join(OUTPUT);
    let before = fs::read(&output).unwrap();
    let modified = fs::metadata(&output).unwrap().modified().unwrap();
    fixture.write(
        "skill/generate-notion-styles/nested/SKILL.md",
        "Invalid output metadata",
    );
    assert!(!update(&fixture.root, false).unwrap());
    assert!(!update(&fixture.root, true).unwrap());
    assert_eq!(before, fs::read(&output).unwrap());
    assert_eq!(modified, fs::metadata(&output).unwrap().modified().unwrap());
    assert_eq!(
        read_document(&output, true).unwrap().0["name"].as_str(),
        Some("generate-notion-styles")
    );
}

#[test]
fn check_reports_missing_and_stale_output_without_writing() {
    let fixture = Fixture::new();
    fixture.skill("example", "example", "Keep the context.");
    let output = fixture.root.join(OUTPUT);
    assert!(
        update(&fixture.root, true)
            .unwrap_err()
            .to_string()
            .contains("is stale")
    );
    assert!(!output.exists());
    assert!(!output.parent().unwrap().exists());
    assert!(update(&fixture.root, false).unwrap());
    let previous = fs::read(&output).unwrap();
    fixture.skill("example", "example", "An added rule.");
    assert!(update(&fixture.root, true).is_err());
    assert_eq!(previous, fs::read(output).unwrap());
}

#[test]
fn expands_metadata_source_and_tracks_guide_edits_and_removal() {
    let fixture = Fixture::new();
    fixture.source_skill("../guide.md");
    let guide = fixture.write("skill/guide.md", "---\nname: example\n---\nActual rules.");
    let output = generate(&fixture.root).unwrap();
    assert!(output.contains("Actual rules."));
    assert!(!output.contains("Read the guide through the CLI."));
    fixture.write("skill/guide.md", "Updated rules without frontmatter.");
    assert!(
        generate(&fixture.root)
            .unwrap()
            .contains("Updated rules without frontmatter.")
    );
    fs::remove_file(guide).unwrap();
    assert!(generate(&fixture.root).is_err());
}

#[test]
fn preserves_multiline_yaml_descriptions_unicode_and_crlf_documents() {
    let fixture = Fixture::new();
    fixture.write("skill/example/SKILL.md", "---\r\nname: example\r\ndescription: >\r\n  Apply to instructions:\r\n  preserve their meaning.\r\n---\r\nTürkçe açıklamayı koru.\r\n");
    let output = generate(&fixture.root).unwrap();
    assert!(output.contains("Apply to instructions: preserve their meaning."));
    assert!(output.contains("Türkçe açıklamayı koru."));
    assert!(!output.contains('\r'));
}

#[test]
fn normalizes_template_and_source_line_endings_without_rewriting_equivalent_output() {
    let fixture = Fixture::new();
    fixture.source_skill("../guide.md");
    fixture.write("skill/guide.md", "First line.\n\nSecond line.");
    let expected = generate(&fixture.root).unwrap();
    fixture.write(
        TEMPLATE,
        &include_str!("generate-skill.template.md").replace('\n', "\r\n"),
    );
    fixture.write("skill/guide.md", "First line.\r\rSecond line.");
    assert_eq!(expected, generate(&fixture.root).unwrap());
    let crlf = expected.replace('\n', "\r\n");
    fixture.write(OUTPUT, &crlf);
    assert!(!update(&fixture.root, true).unwrap());
    assert!(!update(&fixture.root, false).unwrap());
    assert_eq!(crlf, fs::read_to_string(fixture.root.join(OUTPUT)).unwrap());
}

#[test]
fn rejects_duplicate_names_including_the_generated_skill_name() {
    let fixture = Fixture::new();
    fixture.skill("example", "example", "Rules.");
    let duplicate = fixture.skill("duplicate", "example", "Other rules.");
    assert!(
        generate(&fixture.root)
            .unwrap_err()
            .to_string()
            .contains("duplicate skill name")
    );
    fs::remove_file(duplicate).unwrap();
    fixture.skill("collision", "generate-notion-styles", "Rules.");
    assert!(
        generate(&fixture.root)
            .unwrap_err()
            .to_string()
            .contains("duplicate skill name")
    );
}

#[test]
fn rejects_missing_malformed_or_invalid_metadata_without_creating_output() {
    let fixture = Fixture::new();
    for invalid in [
        "Missing frontmatter",
        "---\nname: example",
        "---\n- not-a-mapping\n---\nRule.",
        "---\nname: example\n---\nRule.",
        "---\nname: []\ndescription: Example\n---\nRule.",
        "---\nname: INVALID\ndescription: Example\n---\nRule.",
        "---\nname: example\ndescription: ''\n---\nRule.",
        "---\nname: example\ndescription: [oops\n---\nRule.",
        "---\nname: example\ndescription: Example\nmetadata: []\n---\nRule.",
        "---\nname: example\ndescription: Example\nmetadata:\n  source: []\n---\nRule.",
        "---\nname: example\ndescription: Example\nmetadata:\n  source: ''\n---\nRule.",
        "---\nname: example\ndescription: Example\n---\n  ",
    ] {
        fixture.write("skill/example/SKILL.md", invalid);
        assert!(
            update(&fixture.root, false).is_err(),
            "accepted {invalid:?}"
        );
        assert!(!fixture.root.join(OUTPUT).exists());
    }
}

#[test]
fn rejects_sources_outside_skill_tree_and_self_references() {
    let fixture = Fixture::new();
    for source in [
        "../../outside.md",
        "../generate-notion-styles/SKILL.md",
        "SKILL.md",
    ] {
        fixture.source_skill(source);
        assert!(generate(&fixture.root).is_err(), "accepted {source}");
    }
}

#[test]
fn makes_relative_links_portable_and_preserves_code_examples() {
    let fixture = Fixture::new();
    fixture.skill("example", "example", "# Rules\n[Other](../other.md#notes)\n[Online](https://example.com/a)\n[Ref]: ../other.md\n`[Code](../other.md)`\n```markdown\n# Heading example\n[Code](../other.md)\n```\n");
    let output = generate(&fixture.root).unwrap();
    assert!(output.contains("/blob/main/skill/other.md#notes)"));
    assert!(output.contains("[Ref]: https://github.com/"));
    assert!(output.contains("[Online](https://example.com/a)"));
    assert!(output.contains("`[Code](../other.md)`"));
    assert!(output.contains("```markdown\n# Heading example\n[Code](../other.md)\n```"));
    assert!(output.contains("### Rules"));
}

#[test]
fn preserves_url_queries_fragments_and_external_schemes_and_encodes_paths() {
    let fixture = Fixture::new();
    fixture.skill("türkçe", "example", "[Guide](../çizim.md?q=1#şekil)\n[Anchor](#local)\n[Mail](mailto:hello@example.com)\n[CDN](//example.com/a)\n[Data](data:text/plain,example)");
    let output = generate(&fixture.root).unwrap();
    assert!(output.contains("/skill/t%C3%BCrk%C3%A7e/SKILL.md)"));
    assert!(output.contains("/skill/%C3%A7izim.md?q=1#şekil)"));
    for link in [
        "[Anchor](#local)",
        "[Mail](mailto:hello@example.com)",
        "[CDN](//example.com/a)",
        "[Data](data:text/plain,example)",
    ] {
        assert!(output.contains(link));
    }
}

#[test]
fn orders_sections_by_metadata_name_instead_of_folder_name() {
    let fixture = Fixture::new();
    fixture.skill("first", "zebra", "Last.");
    fixture.skill("last", "alpha", "First.");
    let output = generate(&fixture.root).unwrap();
    assert!(output.find("## alpha").unwrap() < output.find("## zebra").unwrap());
}

#[test]
fn keeps_existing_output_when_source_validation_fails() {
    let fixture = Fixture::new();
    fixture.skill("example", "example", "Rules.");
    update(&fixture.root, false).unwrap();
    let output = fixture.root.join(OUTPUT);
    let before = fs::read(&output).unwrap();
    fixture.write("skill/example/SKILL.md", "Invalid metadata");
    assert!(update(&fixture.root, false).is_err());
    assert_eq!(before, fs::read(output).unwrap());
}

#[test]
fn requires_source_skill_names_to_match_discoverable_names() {
    let fixture = Fixture::new();
    fixture.source_skill("../guide.md");
    fixture.write("skill/guide.md", "---\nname: wrong-guide\n---\nRules.");
    assert!(
        generate(&fixture.root)
            .unwrap_err()
            .to_string()
            .contains("source skill name does not match")
    );
}

#[test]
fn rejects_empty_skill_trees_and_links_outside_repository() {
    let fixture = Fixture::new();
    assert!(
        generate(&fixture.root)
            .unwrap_err()
            .to_string()
            .contains("No source skills")
    );
    fixture.skill("example", "example", "[Outside](../../../outside.md)");
    assert!(generate(&fixture.root).is_err());
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_that_escape_source_and_repository_roots() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let outside = tempdir().unwrap();
    fs::write(outside.path().join("guide.md"), "Outside rules.").unwrap();
    fixture.source_skill("../linked/guide.md");
    symlink(outside.path(), fixture.root.join("skill/linked")).unwrap();
    assert!(generate(&fixture.root).is_err());

    fixture.skill("example", "example", "[Outside](../linked/guide.md)");
    assert!(generate(&fixture.root).is_err());

    fs::remove_file(fixture.root.join("skill/example/SKILL.md")).unwrap();
    symlink(
        outside.path().join("guide.md"),
        fixture.root.join("skill/example/SKILL.md"),
    )
    .unwrap();
    assert!(generate(&fixture.root).is_err());
}
