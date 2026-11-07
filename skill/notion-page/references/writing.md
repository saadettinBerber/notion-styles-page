# Writing pages

A page is one `.md` file: YAML frontmatter followed by Markdown. The [showcase](../templates/showcase.md) demonstrates every supported block.

## Frontmatter

Start each file with a YAML block between `---` lines. The parser supports the field names below; optional metadata keys retain their existing spelling.

```yaml
---
title: Weekly notes
ikon: 🧠
kapak: blue
tarih: 2026-09-27
etiketler: [meeting, planning]
ozet: This week's decisions and open questions.
---
```

| Field | Required | Meaning |
|---|---|---|
| `title` or `baslik` | Yes | Page title. If both appear, `baslik` takes precedence. Start body sections with `##`. |
| `ikon` | No | One emoji above the title. Defaults to 📄. |
| `kapak` | No | A supported color, an image URL (`https://…` or `./image.png`), or `yok` to disable the cover. Omit it for no cover. Unsupported values produce an error. |
| `tarih` | No | Date in `YYYY-MM-DD` format. |
| `etiketler` | No | Tags as `[a, b]` or `a, b`. The same tag keeps the same color across pages. |
| `ozet` | No | One-sentence link-preview summary; not shown in the page body. |
| `font` | No | `daily-dose` (default) or `system`. Unsupported values produce an error. |

Supported colors: `gray`, `brown`, `orange`, `yellow`, `green`, `blue`, `purple`, `pink`, and `red`. Existing Turkish color names are also accepted.

## Typography

The default `daily-dose` preset uses Noto Serif body text and Domine headings, inspired by Daily Dose of Data Science. It supports regular, **bold**, and *italic* text, including Turkish glyphs. Fonts are embedded for offline use. `system` uses the previous system-font appearance. Code keeps its monospace font with either preset.

Set a document's preference in frontmatter:

```yaml
font: system
```

Override it when converting with `notion-page notes.md --font daily-dose`. Precedence is CLI `--font`, then frontmatter `font`, then `daily-dose`. Archive conversion supports the same options. `--restore` returns the original Markdown without changing its font setting.

## Standard Markdown

Supported syntax includes headings, **bold**, *italic*, ~~strikethrough~~, `inline code`, links, images, bullet and numbered lists, `- [ ]` / `- [x]` tasks, blockquotes, horizontal rules, tables, fenced code with language tags, and footnotes (`[^1]`).

Code blocks scroll horizontally to preserve indentation. Fences tagged `text` wrap long lines inside the block; use them for prompts, logs, and other plain text. `==Highlighted text==` adds a yellow highlight.

Readers can also select text in the HTML and choose **Vurgula** to highlight it,
or **Vurguyu kaldır** to remove the selected portion of a reading highlight.
Choose **Vurgulu HTML’i kaydet** to download a copy with marks embedded in the
HTML. Use that copy to reopen or share the page, including in another browser
or when browser storage is blocked. Save again after edits; the open file is not
overwritten automatically. Saved reading marks never change Markdown or
`--restore` output; only archived pages contain restorable Markdown.
Unsaved edits use browser storage for the current file and content when available.
Moving the file, changing its content, or clearing browser storage loses unsaved
edits. Reading highlights work offline in browsers with the CSS Custom Highlight API.

## Notion blocks

Open a container with `:::name` and close it with `:::` on its own line. Containers support Markdown content. A container marker inside a code fence is literal text.

The [document reader](reader.md) rejects unknown or unclosed containers with their original source line numbers.

### Callout

```markdown
:::callout 💡 blue
The main idea goes here.
:::
```

The optional emoji and color can appear in either order. Defaults are 💡 and gray. A misspelled color such as `bluee` is treated as an icon rather than a color, so use a supported name.

### Toggle

```markdown
:::toggle More detail
Initially hidden content.
:::
```

The text after `toggle` is the clickable title and supports **bold** and `code`. `:::toggle-heading` uses the same syntax with an `h1`-sized title. Use it sparingly for a prominent section.

### Columns

```markdown
::::columns
:::column
**Previous approach**

Rules are written by hand.
:::
:::column
**New approach**

Rules are learned from data.
:::
::::
```

Columns stack vertically on phones.

### Video

```markdown
:::video media/demo.mp4 media/demo-poster.jpg
Video caption.
:::
```

- The first path is a video (`.mp4`, `.webm`); the second is an optional poster (`.jpg`, `.png`, `.webp`).
- Paths resolve relative to the Markdown file. A neighboring `media/` directory keeps assets together.
- Playback starts when the reader presses play. Prefer a poster because the first video frame may be blank. Extract one with `ffmpeg -ss 3 -i media/demo.mp4 -frames:v 1 -vf scale=1280:-1 -q:v 3 media/demo-poster.jpg`.
- The container body is an optional caption. Missing files or unsupported media types stop conversion with `✗`.

### Nested containers

Outer containers need more colons than inner ones. Use `::::` around `:::`; add another colon for each enclosing level. Equal fences can close both containers early and leave a visible marker in the output.

```markdown
::::callout 💡 blue
Main idea.

:::toggle Detail
Hidden explanation.
:::
::::
```

## Glossary terms

Define unfamiliar terms once in a `:::terms` block. It can appear anywhere in the
body; placing it at the end keeps definitions together. The block itself has no
visible output.

```markdown
BREX checks project rules. A crate provides reusable Rust code.

:::terms
- **BREX:** Business Rules Exchange: the project's own S1000D rules.
- **Crate:** A Rust library package, such as `serde`.
:::
```

Each item must be one line in the form `- **Term:** definition`. Definitions
support inline Markdown, including emphasis, code, and links. Use at most one
terms block per page and define each term once. Malformed items, duplicate terms,
and a second block stop conversion with the original source line number. An
unused term produces a warning on stderr; the page is still written.

The first occurrence of each term is marked with a dotted underline in each
`##` section, including the content before the first `##`. Matching includes
paragraphs, lists, table cells, callouts, toggle bodies, and blockquotes. Headings,
toggle titles, code, links, raw HTML, figures/SVG, frontmatter, and the glossary
itself are excluded.

Terms match their written spelling. When the second letter is lowercase, the
first letter can also be lowercase: `Crate` matches `crate'leri`. Turkish casing
is respected (`İ` → `i`, `I` → `ı`); acronyms remain case-sensitive. Turkish
suffixes stay inside the marked word, including `BREX'e`, `XSD’nin`, and
`veri modülünü`. A preceding letter or digit prevents a match. When terms
overlap, the longer term wins.

Click or tap a marked word to open its definition; press Esc or click outside to
close it. Tab followed by Enter also opens the card. Only one card stays open.
Cards follow the page's light or dark theme and fit phone screens. Native browser
popovers still work without JavaScript, with the card centered instead of next to
the word. Archive and restore preserve the original terms block exactly.

## Media

| Asset | Embedded? | Notes |
|---|---|---|
| Video and poster in `:::video` | Yes | The page remains one file, with the media added to its size. |
| Inline SVG in `<figure>` | Yes | Included as markup. See the [drawing guide](drawing.md). |
| Image in `kapak` | No | Linked by URL. Relative paths resolve from the output HTML directory. |
| Markdown image `![Description](image.png)` | No, by default | Linked by URL. Library callers can explicitly enable safe embedding. |

When sharing one HTML file, use reachable HTTPS URLs for linked images or draw the visual as inline SVG. With `-o`, relative image paths resolve from the output directory.

The library option `prepared.render(media, embed_images=media.embedded_image_data_uri)` embeds regular PNG, JPEG, GIF, and WebP files within the document directory. Construct `MediaFolder` with that directory. Validation uses actual image bytes rather than extensions and rejects truncated images, SVG, parent traversal, and symlinks. Each `img` retains its original path in `data-source`; repeated uses remain separate elements. This option applies only to body Markdown images. See the [API guide](reader.md) for the preparation order and media contract.
