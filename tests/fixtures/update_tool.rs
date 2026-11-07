// Fake Cargo/Git processes for updater contract tests. No network or real installation.
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let directory = PathBuf::from(env::var_os("UPDATE_FIXTURE").unwrap());
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("commands"))
        .unwrap();
    writeln!(log, "{}", args.join(" ")).unwrap();

    match args[0].as_str() {
        "metadata" => {
            let manifest = env::current_dir().unwrap().join("Cargo.toml");
            println!(
                "{{\"packages\":[{{\"name\":\"notion-sayfa\",\"manifest_path\":{:?}}}]}}",
                manifest.to_str().unwrap()
            );
        }
        "build" => {
            if env::var_os("UPDATE_FAIL_BUILD").is_some() {
                std::process::exit(1);
            }
            println!(
                "{{\"reason\":\"compiler-artifact\",\"target\":{{\"name\":\"notion-page\"}},\"executable\":{:?}}}",
                directory.join("built-cli").to_str().unwrap()
            );
        }
        "install" => {
            let root = PathBuf::from(&args[args.iter().position(|arg| arg == "--root").unwrap() + 1]);
            assert!(root.is_absolute());
            fs::create_dir_all(root.join("bin")).unwrap();
            let staged = root.join("bin/new-cli");
            fs::copy(directory.join("built-cli"), &staged).unwrap();
            fs::rename(
                staged,
                root.join("bin").join(format!("notion-page{}", env::consts::EXE_SUFFIX)),
            ).unwrap();
        }
        "pull" if env::var_os("UPDATE_FAIL_PULL").is_some() => std::process::exit(1),
        "run" | "pull" => {}
        other => panic!("unexpected command {other}"),
    }
}
