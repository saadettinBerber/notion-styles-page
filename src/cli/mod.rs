mod archive;
mod build;
mod skills;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use clap::builder::{PossibleValuesParser, TypedValueParser};
use notion_sayfa::{FontPreset, PageOptions};

const HELP_FOOTER: &str = "\
Examples
  notion-page notes.md
  notion-page notes/*.md --output pages
  notion-page notes.md --font system
  notion-page --archive notes.md
  notion-page --restore notes.html
  notion-page --skill
  notion-page --skill basic-make-notes

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
    /// Markdown files to convert, or HTML archives to restore
    #[arg(
        required_unless_present_any = ["skills", "skill"],
        value_name = "FILES"
    )]
    sources: Vec<PathBuf>,

    /// Write output to this directory (default: next to each source file)
    #[arg(short = 'o', long = "output", alias = "cikti", value_name = "DIR")]
    output_dir: Option<PathBuf>,

    /// Typography preset (default: frontmatter font, then daily-dose)
    #[arg(
        long,
        value_name = "PRESET",
        value_parser = font_parser(),
        conflicts_with = "restore"
    )]
    font: Option<FontPreset>,

    /// Embed the Markdown in the HTML and remove the original after saving
    #[arg(long, conflicts_with = "restore")]
    archive: bool,

    /// Recover the original Markdown from an HTML archive
    #[arg(long)]
    restore: bool,

    /// List the embedded skills and guides
    #[arg(
        long,
        conflicts_with_all = ["skill", "sources", "output_dir", "font", "archive", "restore"]
    )]
    skills: bool,

    /// List skills and guides, or print one by name for an AI agent
    #[arg(
        long,
        value_name = "NAME",
        value_enum,
        num_args = 0..=1,
        default_missing_value = "list",
        conflicts_with_all = ["sources", "output_dir", "font", "archive", "restore"]
    )]
    skill: Option<skills::Skill>,
}

pub(crate) fn run() -> ExitCode {
    let args = Args::parse();
    if args.skills {
        skills::print_list();
        return ExitCode::SUCCESS;
    }
    if let Some(skill) = args.skill {
        skill.print();
        return ExitCode::SUCCESS;
    }
    let mode = if args.archive {
        build::Mode::Archive
    } else if args.restore {
        build::Mode::Restore
    } else {
        build::Mode::Convert
    };
    let options = PageOptions { font: args.font };
    match build::run(
        &args.sources,
        args.output_dir.as_deref(),
        mode,
        &options,
        report,
    ) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("✗ {error}");
            ExitCode::FAILURE
        }
    }
}

fn font_parser() -> impl TypedValueParser<Value = FontPreset> {
    PossibleValuesParser::new(FontPreset::ALL.into_iter().map(FontPreset::as_str))
        .try_map(|value| value.parse::<FontPreset>())
}

fn report(source: &std::path::Path, result: Result<PathBuf, build::BuildError>) {
    let filename = build::display_name(source);
    match result {
        Ok(output) => println!("✓ {filename} → {}", build::display_name(&output)),
        Err(error) => eprintln!("✗ {filename}: {error}"),
    }
}
