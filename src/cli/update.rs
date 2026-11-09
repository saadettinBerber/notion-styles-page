mod skill_sync;

use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(clap::Args)]
pub(super) struct Args {
    /// Source checkout (default: current checkout, then the CLI's source checkout)
    #[arg(long, value_name = "DIR")]
    source: Option<PathBuf>,

    /// Fetch upstream changes with git pull --ff-only before updating
    #[arg(long)]
    pull: bool,

    /// Cargo installation root (default: running CLI's Cargo root, then CARGO_HOME)
    #[arg(long, value_name = "DIR")]
    root: Option<PathBuf>,

    /// Sync an existing skill installation, including missing skills (repeatable, replaces defaults)
    #[arg(long = "skills-dir", value_name = "DIR")]
    skills_dirs: Vec<PathBuf>,
}

pub(super) fn run(args: Args) -> Result<()> {
    let source = source_directory(args.source.as_deref())?;
    let root = installation_root(args.root)?;
    fs::create_dir_all(&root)?;
    let root = fs::canonicalize(root)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(root.join(".notion-page-update.lock"))?;
    lock.try_lock()
        .map_err(|error| format!("could not lock {} for updating: {error}", root.display()))?;

    println!("Source: {}", source.display());
    if args.pull {
        checked(
            Command::new("git")
                .current_dir(&source)
                .args(["pull", "--ff-only"]),
        )?;
    }
    validate_package(&source)?;
    checked(cargo(&source).args(["run", "--locked", "--example", "generate-skill"]))?;

    let roots = if args.skills_dirs.is_empty() {
        default_skill_roots(&source)?
    } else {
        args.skills_dirs
    };
    let skills = skill_sync::plan(&source.join("skill"), &roots)?;
    let built = build(&source)?;
    let installed = root
        .join("bin")
        .join(format!("notion-page{}", env::consts::EXE_SUFFIX));
    if same_contents(&built, &installed)? {
        println!("CLI is up to date: {}", installed.display());
    } else {
        checked(
            cargo(&source)
                .args(["install", "--locked", "--path"])
                .arg(&source)
                .arg("--root")
                .arg(&root)
                .args(["--bin", "notion-page", "--force"]),
        )?;
        println!("Updated CLI: {}", installed.display());
    }
    skills.apply()?;
    println!("CLI update and skill synchronization complete.");
    Ok(())
}

fn is_source(path: &Path) -> bool {
    path.join("Cargo.toml").is_file()
        && path.join("scripts/generate_skill.rs").is_file()
        && path.join("skill/notion-page/SKILL.md").is_file()
}

fn source_directory(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        let path = fs::canonicalize(path)?;
        if !is_source(&path) {
            return Err(format!(
                "{} is not a notion-styles-page source directory",
                path.display()
            )
            .into());
        }
        return Ok(path);
    }
    let current = env::current_dir()?;
    if let Some(path) = current.ancestors().find(|path| is_source(path)) {
        return Ok(fs::canonicalize(path)?);
    }
    let compiled_source = Path::new(env!("CARGO_MANIFEST_DIR"));
    if is_source(compiled_source) {
        return Ok(fs::canonicalize(compiled_source)?);
    }
    // npm's prepare step may compile in a temporary directory before packaging.
    let executable = env::current_exe()?.canonicalize()?;
    if let Some(path) = executable.parent().and_then(Path::parent)
        && is_source(path)
    {
        return Ok(fs::canonicalize(path)?);
    }
    Err("source checkout not found; pass --source /path/to/notion-styles-page".into())
}

fn home_directory() -> Result<PathBuf> {
    env::home_dir().ok_or_else(|| "could not locate your home directory".into())
}

fn installation_root(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    if let Some(path) = env::var_os("CARGO_INSTALL_ROOT") {
        return Ok(path.into());
    }
    let executable = env::current_exe()?.canonicalize()?;
    if let Some(bin) = executable.parent()
        && bin.file_name().is_some_and(|name| name == "bin")
        && let Some(root) = bin.parent()
        && (root.join(".crates2.json").is_file() || root.join(".crates.toml").is_file())
    {
        return Ok(root.to_path_buf());
    }
    Ok(env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or(home_directory()?.join(".cargo")))
}

fn default_skill_roots(source: &Path) -> Result<Vec<PathBuf>> {
    let home = home_directory()?;
    let codex = env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".codex"));
    let claude = env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".claude"));
    let current = env::current_dir()?;
    let mut roots = vec![
        codex.join("skills"),
        claude.join("skills"),
        home.join(".agents/skills"),
    ];
    for project in [&current, source] {
        for directory in [".agents/skills", ".codex/skills", ".claude/skills"] {
            roots.push(project.join(directory));
        }
    }
    Ok(roots)
}

fn cargo(source: &Path) -> Command {
    let mut command = Command::new("cargo");
    command.current_dir(source);
    command
}

fn checked(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        return Err(format!(
            "{} failed ({status})",
            command.get_program().to_string_lossy()
        )
        .into());
    }
    Ok(())
}

fn validate_package(source: &Path) -> Result<()> {
    let output = cargo(source)
        .args(["metadata", "--locked", "--no-deps", "--format-version=1"])
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err("could not read the source package metadata".into());
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let expected = source.join("Cargo.toml");
    let valid = metadata["packages"].as_array().is_some_and(|packages| {
        packages.iter().any(|package| {
            package["name"] == "notion-sayfa"
                && package["manifest_path"]
                    .as_str()
                    .is_some_and(|path| Path::new(path) == expected)
        })
    });
    if !valid {
        return Err(format!(
            "{} does not contain the notion-sayfa package",
            source.display()
        )
        .into());
    }
    Ok(())
}

fn build(source: &Path) -> Result<PathBuf> {
    let output = cargo(source)
        .args([
            "build",
            "--locked",
            "--release",
            "--bin",
            "notion-page",
            "--message-format=json-render-diagnostics",
        ])
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err("CLI build failed; retry after resolving the Cargo error".into());
    }
    for line in output.stdout.split(|&byte| byte == b'\n') {
        if let Ok(message) = serde_json::from_slice::<serde_json::Value>(line)
            && message["reason"] == "compiler-artifact"
            && message["target"]["name"] == "notion-page"
            && let Some(executable) = message["executable"].as_str()
        {
            return Ok(PathBuf::from(executable));
        }
    }
    Err("Cargo did not report the built notion-page executable".into())
}

fn same_contents(source: &Path, target: &Path) -> Result<bool> {
    match fs::read(target) {
        Ok(target) => Ok(fs::read(source)? == target),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
