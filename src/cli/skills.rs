use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub(super) enum Skill {
    #[value(help = "List available skills and guides with descriptions.")]
    List,
    #[value(help = "Convert Markdown notes into standalone Notion-style HTML.")]
    NotionPage,
    #[value(
        alias = "communication",
        help = "Communicate clearly in conversations and documents with context-sensitive Markdown."
    )]
    Simple,
    #[value(help = "Format instructions and checklists as clear grouped rules.")]
    ReadableInstructions,
    #[value(help = "Write concise notes that make sense without the conversation.")]
    BasicMakeNotes,
    #[value(help = "Create Notion-style HTML with shared writing guidance.")]
    GenerateNotionStyles,
    #[value(help = "Create Notion-style HTML and visually verify the result.")]
    GenerateNotionStylesWithVerificate,
    #[value(help = "Inspect existing HTML in light and dark themes and at phone width.")]
    VisualVerification,
    #[value(help = "Write natural Turkish notes with consistent terminology and correct spelling.")]
    TurkishWriting,
    #[value(
        help = "Write source-based notes, independently review them, and render verified HTML."
    )]
    VerifiedNotes,
    #[value(help = "Review existing notes against primary sources without editing files.")]
    VerifiedNotesReviewer,
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
            Self::ReadableInstructions => {
                include_str!("../../skill/readable-instructions/SKILL.md")
            }
            Self::BasicMakeNotes => include_str!("../../skill/basic-make-notes/SKILL.md"),
            Self::GenerateNotionStyles => {
                include_str!("../../skill/generate-notion-styles/SKILL.md")
            }
            Self::GenerateNotionStylesWithVerificate => {
                include_str!("../../skill/generate-notion-styles-with-verificate/SKILL.md")
            }
            Self::VisualVerification => include_str!("../../skill/visual-verification/SKILL.md"),
            Self::TurkishWriting => include_str!("../../skill/turkish-writing/SKILL.md"),
            Self::VerifiedNotes => include_str!("../../skill/verified-notes/SKILL.md"),
            Self::VerifiedNotesReviewer => {
                include_str!("../../skill/verified-notes-reviewer/SKILL.md")
            }
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
