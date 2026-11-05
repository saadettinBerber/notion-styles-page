use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub(super) enum Skill {
    #[value(help = "Kullanılabilir skill ve rehberleri açıklamalarıyla listeler.")]
    List,
    #[value(help = "Markdown notlarını Notion görünümlü, tek dosyalık HTML sayfasına dönüştürür.")]
    NotionPage,
    #[value(help = "Sohbeti hatırlamadan anlaşılabilen, anlam kaybı olmadan öz notlar yazar.")]
    BasicMakeNotes,
    #[value(
        help = "Sayfa başlıklarının, Markdown sözdiziminin ve özel blokların yazımını açıklar."
    )]
    Yazim,
    #[value(help = "Sayfaya gömülü SVG çizimlerin nasıl hazırlanacağını açıklar.")]
    Cizim,
    #[value(help = "Desteklenen blokları örnek bir Markdown sayfasında gösterir.")]
    Vitrin,
    #[value(help = "Rust ve Python API sözleşmelerini ve kullanımını açıklar.")]
    Okuyucu,
}

impl Skill {
    pub(super) fn print(self) {
        let content = match self {
            Self::List => {
                print_list();
                return;
            }
            Self::NotionPage => include_str!("../../skill/notion-sayfa/SKILL.md"),
            Self::BasicMakeNotes => include_str!("../../skill/basic-make-notes.md"),
            Self::Yazim => notion_sayfa::writing_guide(),
            Self::Cizim => include_str!("../../skill/notion-sayfa/references/cizim.md"),
            Self::Vitrin => include_str!("../../skill/notion-sayfa/templates/vitrin.md"),
            Self::Okuyucu => include_str!("../../skill/notion-sayfa/references/okuyucu.md"),
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
