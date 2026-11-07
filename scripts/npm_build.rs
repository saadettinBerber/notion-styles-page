use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn build(root: &Path, target_kind: &str, target_name: &str) -> io::Result<PathBuf> {
    let output = Command::new("cargo")
        .args([
            "build",
            "--locked",
            "--release",
            target_kind,
            target_name,
            "--message-format=json-render-diagnostics",
            "--manifest-path",
        ])
        .arg(root.join("Cargo.toml"))
        .current_dir(root)
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "cargo build failed with {}",
            output.status
        )));
    }
    output
        .stdout
        .split(|&byte| byte == b'\n')
        .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
        .find_map(|message| {
            if message["reason"] == "compiler-artifact" && message["target"]["name"] == target_name
            {
                message["executable"].as_str().map(PathBuf::from)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            io::Error::other(format!("cargo did not report the {target_name} executable"))
        })
}
