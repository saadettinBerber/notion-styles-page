# Bundled fonts

The `daily-dose` typography preset uses Noto Serif for body text (normal and
italic) and Domine for headings. These are unmodified Google Fonts WOFF2 files,
downloaded on 2026-11-05, with variable weights 400–700. Latin and Latin Extended
subsets include Turkish letters. Other scripts use the CSS fallback fonts.

Both families are licensed under the SIL Open Font License 1.1. The full
copyright and license notices are kept in `noto-serif-OFL.txt` and
`domine-OFL.txt`, and included in generated pages that embed these fonts.

`sources.json` records the original download URLs, byte lengths, and SHA-256
checksums. The files came from this Google Fonts stylesheet with a current
Chrome user agent:

<https://fonts.googleapis.com/css2?family=Domine:wght@400..700&family=Noto+Serif:ital,wght@0,400..700;1,400..700&display=swap>

Upstream licenses:

- <https://github.com/google/fonts/blob/main/ofl/notoserif/OFL.txt>
- <https://github.com/google/fonts/blob/main/ofl/domine/OFL.txt>

Font loading is entirely local at render time. No Google Fonts request is made
when converting or opening a page. Only assets used by the selected preset are
embedded; `system` embeds no font files.

## Adding a preset

`src/typography.rs` owns the preset catalogue, canonical names, validation, and
embedded assets. A new preset is added there with its CSS and any font files;
CLI and Python discover the same catalogue without separate name lists or
defaults. Keep presentation rules in a CSS asset, following `assets/daily-dose.css`.
The shared page shell resolves an explicit option, then the document's `font`,
then the default preset. Markdown parsing and media providers do not load fonts.
