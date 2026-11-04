mod build;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

const HELP_FOOTER: &str = "\
Examples
  notion-page notes.md
  notion-page notes/*.md --output pages

Exit Codes
  0  Success
  1  One or more files could not be converted, or output setup failed
  2  Invalid command-line arguments";

#[derive(Parser)]
#[command(
    name = env!("CARGO_BIN_NAME"),
    about = "Convert Markdown into Notion-style HTML",
    version,
    after_help = HELP_FOOTER
)]
struct Args {
    /// Markdown files to convert
    #[arg(required = true, value_name = "FILES")]
    sources: Vec<PathBuf>,

    /// Write HTML to this directory (default: next to each source file)
    #[arg(short = 'o', long = "output", alias = "cikti", value_name = "DIR")]
    output_dir: Option<PathBuf>,
}

pub(crate) fn run() -> ExitCode {
    let args = Args::parse();
    match build::run(&args.sources, args.output_dir.as_deref(), report) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("✗ {error}");
            ExitCode::FAILURE
        }
    }
}

fn report(source: &std::path::Path, result: Result<PathBuf, build::BuildError>) {
    let filename = build::display_name(source);
    match result {
        Ok(output) => println!("✓ {filename} → {}", build::display_name(&output)),
        Err(error) => eprintln!("✗ {filename}: {error}"),
    }
}
