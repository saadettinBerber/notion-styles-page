mod npm_build;

use std::env;
use std::io;
use std::process::{Command, ExitStatus};

fn main() {
    match launch() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("Could not launch notion-page: {error}");
            std::process::exit(1);
        }
    }
}

fn launch() -> io::Result<ExitStatus> {
    let launcher = env::current_exe()?.canonicalize()?;
    let root = launcher
        .ancestors()
        .find(|path| path.join("Cargo.toml").is_file() && path.join("src/main.rs").is_file())
        .ok_or_else(|| io::Error::other("the npm package's Rust sources could not be found"))?;
    let executable = npm_build::build(root, "--bin", "notion-page")?;
    Command::new(executable)
        .args(env::args_os().skip(1))
        .status()
}
