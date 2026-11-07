use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[test]
fn update_handles_retries_unchanged_versions_relative_roots_and_failures() {
    let workspace = tempfile::tempdir().unwrap();
    let workspace = workspace.path().canonicalize().unwrap();
    let source = workspace.join("source checkout");
    let tools = workspace.join("tools");
    let skills = workspace.join("installed skills");
    fs::create_dir_all(&tools).unwrap();
    fs::create_dir_all(source.join("scripts")).unwrap();
    fs::create_dir_all(source.join("skill/notion-page")).unwrap();
    fs::create_dir_all(skills.join("notion-page")).unwrap();
    fs::write(source.join("Cargo.toml"), "fixture").unwrap();
    fs::write(source.join("scripts/generate_skill.rs"), "fixture").unwrap();
    let document = "---\nname: notion-page\n---\nCurrent instructions.\n";
    fs::write(source.join("skill/notion-page/SKILL.md"), document).unwrap();
    let installed_skill = skills.join("notion-page/SKILL.md");
    fs::write(&installed_skill, "Old instructions.").unwrap();
    fs::write(workspace.join("built-cli"), "cli revision one").unwrap();

    let cargo = tools.join(format!("cargo{}", env::consts::EXE_SUFFIX));
    let compilation = Command::new("rustc")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update_tool.rs"))
        .arg("-o")
        .arg(&cargo)
        .output()
        .unwrap();
    assert_success(&compilation);
    fs::copy(
        &cargo,
        tools.join(format!("git{}", env::consts::EXE_SUFFIX)),
    )
    .unwrap();
    let mut paths = vec![tools];
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
    let path = env::join_paths(paths).unwrap();

    let command = || {
        let mut command = Command::new(env!("CARGO_BIN_EXE_notion-page"));
        command
            .current_dir(&workspace)
            .env("PATH", &path)
            .env("UPDATE_FIXTURE", &workspace)
            .args(["update", "--source"])
            .arg(&source)
            .args(["--root", "relative installation", "--skills-dir"])
            .arg(&skills);
        command
    };
    assert_success(&command().output().unwrap());
    let installed_cli = workspace
        .join("relative installation/bin")
        .join(format!("notion-page{}", env::consts::EXE_SUFFIX));
    assert_eq!(
        fs::read_to_string(&installed_cli).unwrap(),
        "cli revision one"
    );
    assert_eq!(fs::read_to_string(&installed_skill).unwrap(), document);
    let skill_modified = fs::metadata(&installed_skill).unwrap().modified().unwrap();
    let cli_modified = fs::metadata(&installed_cli).unwrap().modified().unwrap();
    fs::write(workspace.join("commands"), "").unwrap();

    let unchanged = command().output().unwrap();
    assert_success(&unchanged);
    assert!(String::from_utf8_lossy(&unchanged.stdout).contains("CLI is up to date"));
    assert!(
        !fs::read_to_string(workspace.join("commands"))
            .unwrap()
            .contains("install ")
    );
    assert_eq!(
        fs::metadata(&installed_cli).unwrap().modified().unwrap(),
        cli_modified
    );
    assert_eq!(
        fs::metadata(&installed_skill).unwrap().modified().unwrap(),
        skill_modified
    );

    fs::write(
        workspace.join("built-cli"),
        "cli revision two, same package version",
    )
    .unwrap();
    fs::write(&installed_skill, "Stale instructions.").unwrap();
    let failure = command().env("UPDATE_FAIL_BUILD", "1").output().unwrap();
    assert_eq!(failure.status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(&installed_cli).unwrap(),
        "cli revision one"
    );
    assert_eq!(
        fs::read_to_string(&installed_skill).unwrap(),
        "Stale instructions."
    );

    fs::write(workspace.join("commands"), "").unwrap();
    let pull_failure = command()
        .arg("--pull")
        .env("UPDATE_FAIL_PULL", "1")
        .output()
        .unwrap();
    assert_eq!(pull_failure.status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(workspace.join("commands")).unwrap(),
        "pull --ff-only\n"
    );
    assert_eq!(
        fs::read_to_string(&installed_cli).unwrap(),
        "cli revision one"
    );

    fs::write(workspace.join("commands"), "").unwrap();
    assert_success(&command().arg("--pull").output().unwrap());
    assert!(
        fs::read_to_string(workspace.join("commands"))
            .unwrap()
            .starts_with("pull --ff-only\nmetadata ")
    );
    assert_eq!(
        fs::read_to_string(&installed_cli).unwrap(),
        "cli revision two, same package version"
    );
    assert_eq!(fs::read_to_string(&installed_skill).unwrap(), document);

    #[cfg(unix)]
    {
        let actual_root = workspace.join("actual cargo root");
        let alias_root = workspace.join("other cargo root");
        for root in [&actual_root, &alias_root] {
            fs::create_dir_all(root.join("bin")).unwrap();
            fs::write(root.join(".crates2.json"), "{}").unwrap();
        }
        let actual_binary = actual_root.join("bin/notion-page");
        let alias = alias_root.join("bin/notion-page");
        fs::copy(env!("CARGO_BIN_EXE_notion-page"), &actual_binary).unwrap();
        std::os::unix::fs::symlink(&actual_binary, &alias).unwrap();
        let result = Command::new(&alias)
            .current_dir(&workspace)
            .env("PATH", &path)
            .env("UPDATE_FIXTURE", &workspace)
            .env_remove("CARGO_INSTALL_ROOT")
            .args(["update", "--source"])
            .arg(&source)
            .arg("--skills-dir")
            .arg(&skills)
            .output()
            .unwrap();
        assert_success(&result);
        assert!(fs::symlink_metadata(&alias).unwrap().is_symlink());
        assert_eq!(
            fs::read(&actual_binary).unwrap(),
            fs::read(workspace.join("built-cli")).unwrap()
        );
    }
}

#[test]
fn update_arguments_do_not_accept_conversion_or_skill_options() {
    for arguments in [
        vec!["update", "--skill"],
        vec!["update", "notes.md"],
        vec!["update", "--output", "pages"],
        vec!["update", "--archive"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_notion-page"))
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{output:?}");
    }
    let help = Command::new(env!("CARGO_BIN_EXE_notion-page"))
        .args(["update", "--help"])
        .output()
        .unwrap();
    assert_success(&help);
    assert!(String::from_utf8_lossy(&help.stdout).contains("--pull"));
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
