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

## Update CLI and installed skills

From the source checkout, update the CLI and its installed skills together.

```sh
notion-page update
```

The command regenerates the combined skill, builds the CLI with Cargo, and
replaces the installed executable only when its contents change. It copies
changed skill files and adds missing references. Skills linked to this source
checkout are already current. Existing files absent from the source are kept.

The default skill locations are the user directories for Codex, Claude Code,
and the Skills CLI (`~/.agents/skills`), plus `.codex/skills`, `.claude/skills`,
and `.agents/skills` in the current directory and source root. `CODEX_HOME`
and `CLAUDE_CONFIG_DIR` override their respective user directories. Only
already-installed skills with matching names are updated.

On another computer with a Git clone, fetch its upstream changes and update
both installations in one command.

```sh
notion-page update --pull
```

`--pull` runs `git pull --ff-only` in the source checkout. Git errors stop the
update. Pass a checkout explicitly when running elsewhere.

```sh
notion-page update --source /path/to/notion-styles-page --pull
```

Source lookup checks the current directory and its parents, then the checkout
used to build the CLI, then the source packaged beside an npm executable.
`--root DIR` selects a Cargo installation root. Repeat `--skills-dir DIR` to
select specific skill directories in place of the defaults.

For an older CLI without `update`, or to run through npx, use the local source
package. Rust/Cargo and Node.js/npm must be installed.

```sh
npx --yes --package=. notion-page update
npx --yes --package=. notion-page update --pull
```

The npm prepare step builds a Rust launcher. Each invocation builds and runs
the CLI from the package's current source, reusing Cargo's build cache. The first
run compiles the project. The private package can be run from a local path or
directly from the GitHub source.

```sh
npx --yes --package=git+https://github.com/saadettinBerber/notion-styles-page.git notion-page update
```

For `--pull`, use a Git checkout through `--source`. npm's packaged source does
not include Git checkout metadata.

## Run

From any directory with `notion-page` on your `PATH`.

```sh
printf '%s\n' '---' 'title: Project notes' '---' '' 'First draft.' > notes.md
notion-page notes.md
```

Open `notes.html`.

Select text while reading, then choose **Vurgula** to highlight it in yellow.
Select highlighted text and choose **Vurguyu kaldır** to remove that portion.
Choose **Vurgulu HTML’i kaydet** to download an HTML copy with your highlights
embedded. Open or share that downloaded file to carry the highlights to another
browser or computer, including when browser storage is blocked. Save a new copy
after adding or removing marks; the open file is not overwritten automatically.
The original Markdown archive is preserved, and `--restore` still returns the
exact source without reading marks.

Unsaved edits also stay in this browser's local storage for this file and content
when storage is available. Moving the file, changing its content, or clearing
browser data loses those unsaved edits. This feature requires a browser with the
[CSS Custom Highlight API](https://developer.mozilla.org/en-US/docs/Web/API/CSS_Custom_Highlight_API).
It works offline and never changes Markdown. Regular HTML without an embedded
Markdown archive still cannot be restored with `--restore`.

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
notion-page --skill verified-notes
notion-page --skill verified-notes-reviewer
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
| `verified-notes` | `$verified-notes` | `/verified-notes` |

`verified-notes` writes source-based notes, independently reviews their claims,
applies verified corrections, and then renders HTML. Its reviewer prompt is
included in the skill and available through `notion-page --skill verified-notes-reviewer`.
Ordinary `generate-notion-styles` page creation does not require this review workflow.

For example, use `/simple Explain the result in plain language` in Claude
Code or `$simple Explain the result in plain language` in Codex. The CLI
embeds the same skill files for `notion-page --skill simple` and
`notion-page --skill turkish-writing`.

</details>

## Maintaining the generated skills

Edit `skill/simple/SKILL.md` to change the shared communication principles
and examples. Both generated skills include its full body. Keep the short core principles in the
standalone `notion-page` entry aligned when changing the philosophy.

One Rust generator, `scripts/generate_skill.rs`, produces two independently
installable entrypoints:

| Output | Workflow source | Shared source skills |
|---|---|---|
| `skill/generate-notion-styles/SKILL.md` | `scripts/generate-skill.template.md` | `simple`, `turkish-writing`, `notion-page` |
| `skill/verified-notes/SKILL.md` | `scripts/verified-notes.template.md` | `simple`, `turkish-writing`, `notion-page` |

The verified-notes output also includes the complete
`skill/verified-notes/references/reviewer.md` prompt. Edit that reference to change
the review checklist or report format. Edit its workflow template to change the
writing, review, correction, and conversion sequence.

The `OUTPUTS` list explicitly selects sources for each output, so adding a skill
does not silently make its workflow mandatory in other skills. Source skills
require YAML `name` and `description` fields and a nonempty instruction body.
The generator rejects invalid metadata, duplicate names within an output, and
generated skill inputs. Shared sections are sorted by name. Ordinary relative
Markdown links in shared skills become GitHub links for copied installations.
Register CLI guides separately in `src/cli/skills.rs`; `include_str!` embeds the
same documents so no source checkout is required at runtime.

Generate or check locally with Cargo:

```sh
cargo run --locked --example generate-skill
cargo run --locked --example generate-skill -- --check
cargo test --locked
```

Edit source skills, workflow templates, or the reviewer prompt, then regenerate
both outputs with the same command. Do not hand-edit generated `SKILL.md` files.
`--check` reports missing or stale outputs without writing. All inputs are
validated before any output is replaced; unchanged files are left alone.
Existing agent installations need to be updated to receive changes.

The generator is a development-only Cargo example. It uses the repository's
existing Rust dependencies and does not require Python or install another CLI.

Generation runs explicitly on your machine; it does not require GitHub Actions
and does not run automatically during `cargo build`. Regenerate before building
the CLI to embed the latest guides. The obsolete Python-based Actions workflow
has been removed.
