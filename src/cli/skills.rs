use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub(super) enum Skill {
    #[value(help = "List available skills and guides with descriptions.")]
    List,
    #[value(help = "Convert Markdown notes into standalone Notion-style HTML.")]
    NotionPage,
    #[value(
        alias = "communication",
        alias = "basic-make-notes",
        alias = "readable-instructions",
        help = "Communicate clearly in conversations and documents with context-sensitive Markdown."
    )]
    Simple,
    #[value(help = "Write natural Turkish notes with consistent terminology and correct spelling.")]
    TurkishWriting,
    #[value(
        alias = "yazim",
        help = "Explain frontmatter, Markdown syntax, and custom blocks."
    )]
    Writing,
    #[value(alias = "cizim", help = "Explain how to prepare inline SVG drawings.")]
    Drawing,
    #[value(
        alias = "vitrin",
        help = "Demonstrate supported blocks in a sample Markdown page."
    )]
    Showcase,
    #[value(
        alias = "okuyucu",
        help = "Describe the Rust and Python document APIs."
    )]
    Reader,
}

impl Skill {
    pub(super) fn print(self) {
        let content = match self {
            Self::List => {
                print_list();
                return;
            }
            Self::NotionPage => include_str!("../../skill/notion-page/SKILL.md"),
            Self::Simple => include_str!("../../skill/simple/SKILL.md"),
            Self::TurkishWriting => include_str!("../../skill/turkish-writing/SKILL.md"),
            Self::Writing => notion_sayfa::writing_guide(),
            Self::Drawing => include_str!("../../skill/notion-page/references/drawing.md"),
            Self::Showcase => include_str!("../../skill/notion-page/templates/showcase.md"),
            Self::Reader => include_str!("../../skill/notion-page/references/reader.md"),
        };
        print!("{content}");
    }
}

pub(super) fn print_list() {
    for skill in Skill::value_variants() {
        if matches!(skill, Skill::List) {
            continue;
        }
        let value = skill.to_possible_value().expect("skills are visible");
        let description = value.get_help().expect("each skill has a description");
        println!("{}\t{description}", value.get_name());
    }
}
