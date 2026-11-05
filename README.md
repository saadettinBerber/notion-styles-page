# Notion Styles Page

<picture>
  <source media="(prefers-reduced-motion: reduce)" srcset="docs/showcase.png">
  <img src="docs/readme-hero.gif" alt="A Markdown file becomes a Notion-style page with a callout, toggle, checklist, and code block.">
</picture>

**Markdown → Notion-style HTML.** One file. Opens offline.

I built this to give notes from any AI agent a consistent format, with the
Notion-style layout I like.

<a id="kurulum"></a>

## Install

```sh
git clone https://github.com/saadettinBerber/notion-styles-page.git
cd notion-styles-page
cargo install --locked --path .
```

## Run

From any directory with `notion-page` on your `PATH`.

```sh
printf '%s\n' '---' 'title: Project notes' '---' '' 'First draft.' > notes.md
notion-page notes.md
```

Open `notes.html`.

The default `daily-dose` typography uses Noto Serif for text and Domine for
headings, inspired by Daily Dose of Data Science. Fonts are embedded in the
HTML and work offline. Use the previous system font appearance with:

```sh
notion-page notes.md --font system
```

Set `font: daily-dose` or `font: system` in the Markdown frontmatter to keep a
document's choice. `--font` overrides frontmatter; without either setting,
`daily-dose` is used. Code keeps its monospace font.

To keep only the HTML file, use archive mode. It embeds the exact Markdown in
the HTML and removes the `.md` file after saving the archive. Restore the
original Markdown whenever you need to edit it.

```sh
notion-page --archive notes.md
notion-page --restore notes.html
```

Both commands refuse to overwrite an existing output. If `notes.html` already
exists, run `notion-page --archive notes.md --output archives`. Regular
conversions also refuse to overwrite an HTML archive. Only HTML created with
`--archive` supports `--restore`. Anyone with a copy of the archive can extract
the source Markdown.

```sh
notion-page --help
notion-page --version
```

Agents can discover the embedded skills and guides, then read one by name.
Each list entry includes a one-sentence description. The source checkout is
not required.

```sh
notion-page --skill
notion-page --skill basic-make-notes
notion-page --skill readable-instructions
notion-page --skill turkish-writing
notion-page --skill writing
```

`basic-make-notes` keeps notes understandable without the conversation and
concise without losing meaning.

`generate-notion-styles` lists guides with `notion-page --skill`, selects them
by name and description, then reads the relevant guides. `basic-make-notes`
also links to `readable-instructions` for procedural notes. Each guide keeps
its own rules in one source file.

`turkish-writing` explains Turkish sentence structure, natural phrasing,
technical terminology, and spelling, drawing on the earlier Oku research.

[Markdown guide](skill/notion-page/references/writing.md) · [Türkçe yazım rehberi](skill/notion-page/references/turkish-writing.md) · [Python API](skill/notion-page/references/reader.md)

<details>
<summary>Ajanlara ekleme</summary>

Install `notion-page` first. With Node.js 22.20+ and Git installed, use the
[Skills CLI](https://github.com/vercel-labs/skills/blob/main/docs/commands.md)
in your terminal on macOS, Linux, or Windows:

**[Codex](https://learn.chatgpt.com/docs/build-skills)**

```sh
npx skills add saadettinBerber/notion-styles-page --skill generate-notion-styles --agent codex --global --copy
```

`$generate-notion-styles Turn these notes into a page.`

**[Claude Code](https://code.claude.com/docs/en/skills)**

```sh
npx skills add saadettinBerber/notion-styles-page --skill generate-notion-styles --agent claude-code --global --copy
```

`/generate-notion-styles Turn these notes into a page.`

These commands copy the skill into each agent's user directory. Omit `--global`
to install only in the current project.

For a standalone skill, replace `generate-notion-styles` in the install command
with one of these names. All entries read their guide through the installed CLI.

| Skill | Codex invocation | Claude Code invocation |
|---|---|---|
| `basic-make-notes` | `$basic-make-notes` | `/basic-make-notes` |
| `readable-instructions` | `$readable-instructions` | `/readable-instructions` |
| `turkish-writing` | `$turkish-writing` | `/turkish-writing` |

</details>
