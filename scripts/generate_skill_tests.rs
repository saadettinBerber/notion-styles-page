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
        fixture.write(
            OUTPUTS[0].template,
            include_str!("generate-skill.template.md"),
        );
        fixture.write(
            OUTPUTS[1].template,
            include_str!("verified-notes.template.md"),
        );
        fixture.write(
            OUTPUTS[2].template,
            include_str!("generate-skill-with-verificate.template.md"),
        );
        fixture.write(
            OUTPUTS[0].workflow.unwrap(),
            include_str!("generate-page-workflow.md"),
        );
        fixture.write(
            OUTPUTS[2].references[0],
            "## Visual verification\n\nInspect HTML.",
        );
        for (folder, body) in [
            ("simple", "Shared clarity."),
            ("turkish-writing", "Türkçe açıklama."),
            ("notion-page", "Convert the page."),
        ] {
            fixture.skill(folder, folder, body);
        }
        fixture.write(
            OUTPUTS[1].references[0],
            "## Reviewer prompt template\n\nIndependent review.",
        );
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

    fn generate(&self, index: usize) -> String {
        generate(&self.root, &OUTPUTS[index]).unwrap()
    }

    fn update(&self, check: bool) -> Result<bool> {
        update(&self.root, OUTPUTS, check)
    }
}

#[test]
fn generates_separate_workflows_from_explicit_shared_sources() {
    let fixture = Fixture::new();
    fixture.skill("unrelated", "unrelated", "Do not include this workflow.");
    let page = fixture.generate(0);
    let verified = fixture.generate(1);
    let visual = fixture.generate(2);
    for output in [&page, &verified, &visual] {
        assert!(output.contains("Shared clarity."));
        assert!(output.contains("Türkçe açıklama."));
        assert!(output.contains("Convert the page."));
        assert!(!output.contains("Do not include this workflow."));
    }
    assert!(!page.contains("Independent review."));
    assert!(!page.contains("## 1. Reopen the primary sources"));
    assert!(verified.contains("Independent review."));
    assert!(verified.contains("## 1. Reopen the primary sources"));
    fixture.update(false).unwrap();
    for spec in OUTPUTS {
        let expected_name = read_document(&fixture.root.join(spec.template), true)
            .unwrap()
            .0["name"]
            .clone();
        assert_eq!(
            read_document(&fixture.root.join(spec.output), true)
                .unwrap()
                .0["name"],
            expected_name
        );
    }
}

#[test]
fn shared_edits_update_all_outputs_and_reviewer_edits_update_only_verified_notes() {
    let fixture = Fixture::new();
    fixture.update(false).unwrap();
    let page = fixture.generate(0);
    let visual = fixture.generate(2);
    fixture.write(OUTPUTS[1].references[0], "Updated reviewer prompt.");
    assert_eq!(page, fixture.generate(0));
    assert_eq!(visual, fixture.generate(2));
    assert!(fixture.generate(1).contains("Updated reviewer prompt."));
    assert!(fixture.update(true).is_err());
    fixture.skill("simple", "simple", "Changed shared guidance.");
    assert!(fixture.update(false).unwrap());
    for spec in OUTPUTS {
        assert!(
            read_text(&fixture.root.join(spec.output))
                .unwrap()
                .contains("Changed shared guidance.")
        );
    }
    assert!(!fixture.update(true).unwrap());
}

#[test]
fn visual_variant_only_appends_verification_to_the_shared_page_body() {
    let fixture = Fixture::new();
    fixture.update(false).unwrap();
    let body = |index: usize| {
        read_document(&fixture.root.join(OUTPUTS[index].output), true)
            .unwrap()
            .1
    };
    let page = body(0);
    let reference = read_text(&fixture.root.join(OUTPUTS[2].references[0])).unwrap();
    assert_eq!(body(2), format!("{page}\n\n---\n\n{}", reference.trim()));

    let verified = fixture.generate(1);
    fixture.write(OUTPUTS[0].workflow.unwrap(), "Updated common workflow.");
    fixture.update(false).unwrap();
    assert_ne!(page, body(0));
    assert_eq!(
        body(2),
        format!("{}\n\n---\n\n{}", body(0), reference.trim())
    );
    assert_eq!(verified, fixture.generate(1));

    let page = fixture.generate(0);
    fixture.write(OUTPUTS[2].references[0], "Updated visual verification.");
    assert_eq!(page, fixture.generate(0));
    assert_eq!(verified, fixture.generate(1));
    assert!(fixture.update(true).is_err());
}

#[test]
fn rejects_missing_empty_or_outside_workflows() {
    let fixture = Fixture::new();
    let mut spec = SkillOutput {
        output: OUTPUTS[0].output,
        template: OUTPUTS[0].template,
        workflow: Some("scripts/missing.md"),
        sources: SHARED_SOURCES,
        references: &[],
    };
    assert!(generate(&fixture.root, &spec).is_err());
    fixture.write("scripts/empty.md", "  ");
    spec.workflow = Some("scripts/empty.md");
    assert!(generate(&fixture.root, &spec).is_err());
    spec.workflow = Some("../outside.md");
    assert!(generate(&fixture.root, &spec).is_err());
}

#[test]
fn missing_and_stale_outputs_are_reported_without_writing() {
    let fixture = Fixture::new();
    let error = fixture.update(true).unwrap_err().to_string();
    for spec in OUTPUTS {
        assert!(error.contains(spec.output));
        assert!(!fixture.root.join(spec.output).exists());
    }
    fixture.update(false).unwrap();
    let old = fs::read(fixture.root.join(OUTPUTS[1].output)).unwrap();
    fixture.write(OUTPUTS[1].references[0], "Changed review.");
    let error = fixture.update(true).unwrap_err().to_string();
    assert!(error.contains(OUTPUTS[1].output));
    assert!(!error.contains(OUTPUTS[0].output));
    assert_eq!(old, fs::read(fixture.root.join(OUTPUTS[1].output)).unwrap());
}

#[test]
fn unchanged_outputs_are_not_rewritten_and_line_endings_are_normalized() {
    let fixture = Fixture::new();
    fixture.update(false).unwrap();
    for spec in OUTPUTS {
        let output = fixture.root.join(spec.output);
        let crlf = read_text(&output).unwrap().replace('\n', "\r\n");
        fixture.write(spec.output, &crlf);
        let modified = fs::metadata(&output).unwrap().modified().unwrap();
        let template = read_text(&fixture.root.join(spec.template)).unwrap();
        fixture.write(spec.template, &template.replace('\n', "\r\n"));
        assert!(!fixture.update(true).unwrap());
        assert!(!fixture.update(false).unwrap());
        assert_eq!(modified, fs::metadata(&output).unwrap().modified().unwrap());
        assert_eq!(crlf, fs::read_to_string(output).unwrap());
    }
}

#[test]
fn validates_all_inputs_before_replacing_any_output() {
    let fixture = Fixture::new();
    fixture.update(false).unwrap();
    let before: Vec<_> = OUTPUTS
        .iter()
        .map(|spec| fs::read(fixture.root.join(spec.output)).unwrap())
        .collect();
    fixture.skill("simple", "simple", "Pending shared change.");
    fs::remove_file(fixture.root.join(OUTPUTS[1].references[0])).unwrap();
    assert!(fixture.update(false).is_err());
    for (spec, bytes) in OUTPUTS.iter().zip(before) {
        assert_eq!(bytes, fs::read(fixture.root.join(spec.output)).unwrap());
    }
}

#[test]
fn rejects_missing_malformed_empty_and_duplicate_sources() {
    let fixture = Fixture::new();
    for invalid in [
        "Missing frontmatter",
        "---\nname: simple",
        "---\n- not-a-mapping\n---\nRule.",
        "---\nname: simple\n---\nRule.",
        "---\nname: []\ndescription: Example\n---\nRule.",
        "---\nname: INVALID\ndescription: Example\n---\nRule.",
        "---\nname: simple\ndescription: ''\n---\nRule.",
        "---\nname: simple\ndescription: [oops\n---\nRule.",
        "---\nname: simple\ndescription: Example\nmetadata: []\n---\nRule.",
        "---\nname: simple\ndescription: Example\nmetadata:\n  source: []\n---\nRule.",
        "---\nname: simple\ndescription: Example\n---\n  ",
    ] {
        fixture.write("skill/simple/SKILL.md", invalid);
        assert!(fixture.update(false).is_err(), "accepted {invalid:?}");
        assert!(!fixture.root.join(OUTPUTS[0].output).exists());
    }
    for name in [
        "notion-page",
        "generate-notion-styles",
        "verified-notes",
        "generate-notion-styles-with-verificate",
    ] {
        fixture.skill("simple", name, "Rules.");
        assert!(
            fixture
                .update(false)
                .unwrap_err()
                .to_string()
                .contains("duplicate skill name")
        );
    }
    fs::remove_file(fixture.root.join("skill/simple/SKILL.md")).unwrap();
    assert!(fixture.update(false).is_err());
}

#[test]
fn resolves_metadata_sources_and_rejects_self_generated_or_outside_sources() {
    let fixture = Fixture::new();
    let source_skill = |source: &str| {
        fixture.write(
            "skill/simple/SKILL.md",
            &format!(
                "---\nname: simple\ndescription: Example\nmetadata:\n  source: {source}\n---\nStub."
            ),
        );
    };
    source_skill("../guide.md");
    fixture.write("skill/guide.md", "Actual source guidance.");
    assert!(fixture.generate(0).contains("Actual source guidance."));
    fixture.write("skill/guide.md", "---\nname: wrong\n---\nWrong source.");
    assert!(fixture.update(false).is_err());
    for source in [
        "../../outside.md",
        "../generate-notion-styles/SKILL.md",
        "../verified-notes/SKILL.md",
        "../generate-notion-styles-with-verificate/SKILL.md",
        "SKILL.md",
    ] {
        source_skill(source);
        assert!(fixture.update(false).is_err(), "accepted {source}");
    }
}

#[test]
fn preserves_yaml_unicode_and_portable_links_without_rewriting_code() {
    let fixture = Fixture::new();
    fixture.write("skill/simple/SKILL.md", "---\r\nname: simple\r\ndescription: >\r\n  Apply to instructions:\r\n  preserve their meaning.\r\n---\r\n# Rules\r\nTürkçe açıklama.\r\n[Guide](../çizim.md?q=1#şekil)\r\n[Ref]: ../other.md\r\n[Mail](mailto:hello@example.com)\r\n[Anchor](#local)\r\n`[Code](../other.md)`\r\n```markdown\r\n# Heading example\r\n[Code](../other.md)\r\n```\r\n");
    let output = fixture.generate(0);
    for expected in [
        "Apply to instructions: preserve their meaning.",
        "Türkçe açıklama.",
        "### Rules",
        "/skill/%C3%A7izim.md?q=1#şekil)",
        "[Ref]: https://github.com/",
        "[Mail](mailto:hello@example.com)",
        "[Anchor](#local)",
        "`[Code](../other.md)`",
        "```markdown\n# Heading example\n[Code](../other.md)\n```",
    ] {
        assert!(output.contains(expected), "missing {expected}");
    }
    assert!(!output.contains('\r'));
    assert!(output.find("## notion-page").unwrap() < output.find("## simple").unwrap());
    fixture.skill("simple", "simple", "[Outside](../../../outside.md)");
    assert!(fixture.update(false).is_err());
}

#[test]
fn rejects_generated_inputs_empty_source_lists_and_empty_references() {
    let fixture = Fixture::new();
    fixture.update(false).unwrap();
    let mut spec = SkillOutput {
        output: "skill/example/SKILL.md",
        template: OUTPUTS[0].template,
        workflow: None,
        sources: &["skill/verified-notes/SKILL.md"],
        references: &[],
    };
    assert!(generate(&fixture.root, &spec).is_err());
    spec.sources = &[];
    assert!(
        generate(&fixture.root, &spec)
            .unwrap_err()
            .to_string()
            .contains("No source skills")
    );
    fixture.write(OUTPUTS[1].references[0], "  ");
    assert!(fixture.update(false).is_err());
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_that_escape_source_and_repository_roots() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let outside = tempdir().unwrap();
    fs::write(outside.path().join("guide.md"), "Outside rules.").unwrap();
    symlink(outside.path(), fixture.root.join("skill/linked")).unwrap();
    fixture.skill("simple", "simple", "[Outside](../linked/guide.md)");
    assert!(fixture.update(false).is_err());
    fixture.skill("simple", "simple", "Shared rules.");
    fs::remove_file(fixture.root.join(OUTPUTS[1].references[0])).unwrap();
    symlink(
        outside.path().join("guide.md"),
        fixture.root.join(OUTPUTS[1].references[0]),
    )
    .unwrap();
    assert!(fixture.update(false).is_err());
    fs::remove_file(fixture.root.join("skill/simple/SKILL.md")).unwrap();
    symlink(
        outside.path().join("guide.md"),
        fixture.root.join("skill/simple/SKILL.md"),
    )
    .unwrap();
    assert!(fixture.update(false).is_err());
}
