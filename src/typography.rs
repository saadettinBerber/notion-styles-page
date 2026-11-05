use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::PageError;

/// A page's font families and reading rhythm.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(try_from = "String", into = "String")]
pub enum FontPreset {
    #[default]
    DailyDose,
    System,
}

impl FontPreset {
    /// The catalogue used by command-line and language bindings.
    pub const ALL: [Self; 2] = [Self::DailyDose, Self::System];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DailyDose => "daily-dose",
            Self::System => "system",
        }
    }

    pub(crate) fn stylesheet(self) -> &'static str {
        match self {
            Self::DailyDose => &DAILY_DOSE_STYLES,
            Self::System => "",
        }
    }
}

impl fmt::Display for FontPreset {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(self.as_str())
    }
}

impl FromStr for FontPreset {
    type Err = PageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|preset| preset.as_str() == value)
            .ok_or_else(|| {
                let choices = Self::ALL.map(Self::as_str).join(", ");
                PageError::Document(format!(
                    "Bilinmeyen yazı tipi: '{value}'. Geçerli seçenekler: {choices}."
                ))
            })
    }
}

impl TryFrom<String> for FontPreset {
    type Error = PageError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<FontPreset> for String {
    fn from(value: FontPreset) -> Self {
        value.to_string()
    }
}

const LATIN: &str = "U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD";
const LATIN_EXT: &str = "U+0100-02BA, U+02BD-02C5, U+02C7-02CC, U+02CE-02D7, U+02DD-02FF, U+0304, U+0308, U+0329, U+1D00-1DBF, U+1E00-1E9F, U+1EF2-1EFF, U+2020, U+20A0-20AB, U+20AD-20C4, U+2113, U+2C60-2C7F, U+A720-A7FF";

struct EmbeddedFont {
    family: &'static str,
    style: &'static str,
    unicode_range: &'static str,
    bytes: &'static [u8],
}

const DAILY_DOSE_FONTS: &[EmbeddedFont] = &[
    EmbeddedFont {
        family: "Domine",
        style: "normal",
        unicode_range: LATIN,
        bytes: include_bytes!("../assets/fonts/domine-latin.woff2"),
    },
    EmbeddedFont {
        family: "Domine",
        style: "normal",
        unicode_range: LATIN_EXT,
        bytes: include_bytes!("../assets/fonts/domine-latin-ext.woff2"),
    },
    EmbeddedFont {
        family: "Noto Serif",
        style: "normal",
        unicode_range: LATIN,
        bytes: include_bytes!("../assets/fonts/noto-serif-latin.woff2"),
    },
    EmbeddedFont {
        family: "Noto Serif",
        style: "normal",
        unicode_range: LATIN_EXT,
        bytes: include_bytes!("../assets/fonts/noto-serif-latin-ext.woff2"),
    },
    EmbeddedFont {
        family: "Noto Serif",
        style: "italic",
        unicode_range: LATIN,
        bytes: include_bytes!("../assets/fonts/noto-serif-italic-latin.woff2"),
    },
    EmbeddedFont {
        family: "Noto Serif",
        style: "italic",
        unicode_range: LATIN_EXT,
        bytes: include_bytes!("../assets/fonts/noto-serif-italic-latin-ext.woff2"),
    },
];

static DAILY_DOSE_STYLES: LazyLock<String> = LazyLock::new(|| {
    use std::fmt::Write;

    let mut css = format!(
        "/*\nDomine\n{}\nNoto Serif\n{}\n*/\n",
        include_str!("../assets/fonts/domine-OFL.txt"),
        include_str!("../assets/fonts/noto-serif-OFL.txt"),
    );
    for font in DAILY_DOSE_FONTS {
        writeln!(
            css,
            "@font-face {{ font-family: '{}'; font-style: {}; font-weight: 400 700; font-display: swap; src: url(data:font/woff2;base64,{}) format('woff2'); unicode-range: {}; }}",
            font.family,
            font.style,
            STANDARD.encode(font.bytes),
            font.unicode_range,
        )
        .expect("writing to a String cannot fail");
    }
    css.push_str(include_str!("../assets/daily-dose.css"));
    css
});
