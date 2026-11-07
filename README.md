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

Define unfamiliar vocabulary once with a `:::terms` block. Each section's first
prose occurrence becomes a dotted word that opens a definition card by click,
tap, or keyboard. Definitions support inline Markdown; cards work offline in
light and dark themes. See the [glossary syntax](skill/notion-page/references/writing.md#glossary-terms)
and [showcase](skill/notion-page/templates/showcase.md).

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

Human, AI, and SI (superintelligence) agents can discover the embedded skills
and guides through the CLI. Each list entry includes a one-sentence description,
and the guides are available outside the source checkout.

```sh
notion-page --skill
notion-page --skill simple
notion-page --skill turkish-writing
notion-page --skill writing
```

The [simple skill](skill/simple/SKILL.md) contains its full instructions. It covers
conversation, progress updates, explanations, final replies, and standalone notes.
It gives the model criteria for choosing paragraphs, lists, tables, and emphasis,
with examples for different reader needs.

`generate-notion-styles` includes these principles and examples directly in its
body, alongside the conversion and language guides. Loading the skill makes
the communication guidance available immediately. The installed skills and
CLI provide the instructions needed for the workflow.

The guides are written in English. The Turkish writing guide includes Turkish
language examples with English explanations. Follow the user's requested
language when preparing replies and documents.

The former `communication`, `basic-make-notes`, and `readable-instructions` CLI names are aliases
for `simple`. Their standalone skill entries have been consolidated into
the simple skill.

`turkish-writing` explains Turkish sentence structure, natural phrasing,
technical terminology, and spelling, drawing on the earlier Oku research.

[Markdown guide](skill/notion-page/references/writing.md) · [Turkish writing guide](skill/turkish-writing/SKILL.md) · [Python API](skill/notion-page/references/reader.md)

<details>
<summary>Add to agents</summary>

Install `notion-page` for page conversion and CLI guides. The `simple`
and `turkish-writing` skills carry their guidance directly. With Node.js 22.20+
and Git installed, use the
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

Each skill can be invoked independently. Use its name in the install command
above. Each `SKILL.md` contains its instructions directly.

| Skill | Codex invocation | Claude Code invocation |
|---|---|---|
| `simple` | `$simple` | `/simple` |
| `turkish-writing` | `$turkish-writing` | `/turkish-writing` |
| `notion-page` | `$notion-page` | `/notion-page` |
| `generate-notion-styles` | `$generate-notion-styles` | `/generate-notion-styles` |

For example, use `/simple Explain the result in plain language` in Claude
Code or `$simple Explain the result in plain language` in Codex. The CLI
embeds the same skill files for `notion-page --skill simple` and
`notion-page --skill turkish-writing`.

</details>

## Maintaining the combined skill

Edit `skill/simple/SKILL.md` to change the shared communication principles
and examples. The generator discovers it with the other skills and includes its
full body in the combined output. Keep the short core principles in the
standalone `notion-page` entry aligned when changing the philosophy.

Add a folder containing `SKILL.md` anywhere under `skill/`. Give it YAML
frontmatter with `name` and `description`, followed by its instructions:

```markdown
---
name: example-guide
description: Explain when these instructions apply.
---

Write the rules here.
```

The generator discovers skills automatically. It excludes
`skill/generate-notion-styles/`, sorts sections by name, and rejects missing
metadata or duplicate names. Keep each skill's full instructions in its
`SKILL.md` so they are available when the skill is loaded.

The combined output uses each skill's description and body. Ordinary relative
Markdown links become GitHub links so copied installations can reach supporting
documents. Register CLI guides separately in `src/cli/skills.rs`, embedding the
same `SKILL.md` that holds their instructions.

Generate or check locally with Cargo:

```sh
cargo run --locked --example generate-skill
cargo run --locked --example generate-skill -- --check
cargo test --locked
```

Edit source skills or `scripts/generate-skill.template.md`, then regenerate;
do not hand-edit the generated `SKILL.md`. `--check` reports stale output without
writing. Existing agent installations need to be updated to receive changes.

The generator is a development-only Cargo example. It uses the repository's
existing Rust dependencies and does not require Python or install another CLI.

GitHub Actions automation is pending publication. The workflow was tested
locally but could not be pushed because the current GitHub session lacks the
`workflow` scope. Use the local generation commands above until it is published.
