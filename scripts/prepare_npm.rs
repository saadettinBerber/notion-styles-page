mod npm_build;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    match prepare() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Could not prepare notion-page for npm: {error}");
            ExitCode::FAILURE
        }
    }
}

fn prepare() -> io::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let executable = npm_build::build(root, "--example", "npm-launcher")?;

    let directory = root.join("npm-bin");
    // The stable .exe name also lets npm's Windows command shim launch the binary.
    let destination = directory.join("notion-page.exe");
    let contents = fs::read(&executable)?;
    match fs::read(&destination) {
        Ok(existing) if existing == contents => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    fs::create_dir_all(&directory)?;
    let mut temporary = tempfile::NamedTempFile::new_in(&directory)?;
    temporary.write_all(&contents)?;
    temporary
        .as_file()
        .set_permissions(fs::metadata(executable)?.permissions())?;
    temporary
        .persist(destination)
        .map_err(|error| error.error)?;
    Ok(())
}
