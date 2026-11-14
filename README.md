# Notion Styles Page

<picture>
  <source media="(prefers-reduced-motion: reduce)" srcset="docs/showcase.png">
  <img src="docs/readme-hero.gif" alt="A Markdown file becomes a Notion-style page with a callout, toggle, checklist, and code block.">
</picture>

**Markdown → Notion-style HTML.** One file. Opens offline.

Give notes from any AI agent a consistent layout, with callouts, toggles,
checklists, code blocks, and embedded fonts.

<a id="kurulum"></a>

## Install

---

**Install the CLI with Rust and Cargo.**

```sh
git clone https://github.com/saadettinBerber/notion-styles-page.git
cd notion-styles-page
cargo install --locked --path .
```

## Run

---

**Create a Markdown file, convert it, and open `notes.html`.**

With `notion-page` on your `PATH`, run from any directory:

```sh
printf '%s\n' '---' 'title: Project notes' '---' '' 'First draft.' > notes.md
notion-page notes.md
```

[Markdown guide](skill/notion-page/references/writing.md) · [Showcase](skill/notion-page/templates/showcase.md) · [Python API](skill/notion-page/references/reader.md)

<details>
<summary>Update CLI and installed skills</summary>

**Run from the source checkout to update the CLI and its installed skills together.**

```sh
notion-page update
notion-page update --pull  # Fetch upstream changes first
```

- **Choose the source:**

  - `--pull` runs `git pull --ff-only`; Git errors stop the update.

  - From elsewhere, use `notion-page update --source /path/to/notion-styles-page --pull`.

  - Automatic lookup checks the current directory and parents, the checkout used to build the CLI, then source packaged beside an npm executable.

- **Choose the installation:**

  - Defaults cover user skill directories for Codex, Claude Code, and the Skills CLI (`~/.agents/skills`), plus `.codex/skills`, `.claude/skills`, and `.agents/skills` in the current directory and source root. `CODEX_HOME` and `CLAUDE_CONFIG_DIR` override their user directories.

  - Repeat `--skills-dir DIR` to replace those defaults. Only directories already containing a repository skill are updated; use **Add to agents** for a first installation.

  - Use `--root DIR` to select a Cargo installation root.

- **What gets updated:**

  - The command regenerates skills, builds the CLI, and replaces changed files. It adds missing repository skills, references, and templates; links to this checkout and files absent from the source are preserved.

  - When upgrading from a CLI that only refreshes existing skills, run the update again to install missing skills.

**Older CLI or npx:** install Rust/Cargo and Node.js/npm, then run from the checkout:

```sh
npx --yes --package=. notion-page update
npx --yes --package=. notion-page update --pull
```

Or run directly from GitHub source:

```sh
npx --yes --package=git+https://github.com/saadettinBerber/notion-styles-page.git notion-page update
```

The private npm package builds a Rust launcher during preparation. Each invocation
builds and runs the current source using Cargo's cache; the first run compiles it.
For `--pull`, pass a Git checkout with `--source`: npm packages lack Git metadata.

</details>

<details>
<summary>Add to agents</summary>

**Install a skill, then invoke it by name in your agent.**

Install `notion-page` for conversion and CLI guides. With Node.js 22.20+ and Git,
use the [Skills CLI](https://github.com/vercel-labs/skills/blob/main/docs/commands.md)
on macOS, Linux, or Windows:

```sh
# Codex
npx skills add saadettinBerber/notion-styles-page --skill generate-notion-styles --agent codex --global --copy

# Claude Code
npx skills add saadettinBerber/notion-styles-page --skill generate-notion-styles --agent claude-code --global --copy
```

- Omit `--global` to install only in the current project.

- Replace `generate-notion-styles` with any skill below. Each includes its own instructions.

- Invoke with `$skill-name` in [Codex](https://learn.chatgpt.com/docs/build-skills) or `/skill-name` in [Claude Code](https://code.claude.com/docs/en/skills), followed by your request.

| Skill | Use it to… |
|---|---|
| `simple` | Write concise replies, explanations, and notes |
| `readable-instructions` | Format instructions and checklists for scanning |
| `basic-make-notes` | Write self-contained notes |
| `turkish-writing` | Use natural Turkish phrasing, terminology, and spelling |
| `notion-page` | Convert Markdown into HTML |
| `generate-notion-styles` | Write and convert notes without post-render visual checks |
| `generate-notion-styles-with-verificate` | Write, convert, and check light/dark themes and phone layout |
| `visual-verification` | Check an existing HTML page |
| `verified-notes` | Write source-based notes, independently review claims, correct, render, and visually check |
| `verified-notes-reviewer` | Review an existing note without editing it |

Example: `$generate-notion-styles Turn these notes into a page.` in Codex,
or `/simple Explain the result in plain language` in Claude Code.
Both page-generation variants fix conversion errors; source review is specific
to `verified-notes`. Guides are in English, with Turkish examples in the
[Turkish writing guide](skill/turkish-writing/SKILL.md); follow the user's requested output language.

</details>

<details>
<summary>Highlight text while reading</summary>

**Save an HTML copy to keep and share your highlights.**

1. Select text and choose **Vurgula** to highlight it in yellow.
2. Select highlighted text and choose **Vurguyu kaldır** to remove that portion.
3. Choose **Vurgulu HTML’i kaydet** to download a copy with highlights embedded.

Save a new copy after each set of changes; the open file is not overwritten.
Downloaded copies carry highlights across browsers and computers, even when
browser storage is blocked. Unsaved edits use local storage when available;
moving the file, changing its content, or clearing browser data loses them.

Highlighting works offline, requires the
[CSS Custom Highlight API](https://developer.mozilla.org/en-US/docs/Web/API/CSS_Custom_Highlight_API),
and never changes Markdown. An embedded Markdown archive stays intact;
`--restore` returns the exact source without reading marks. Highlighting does
not make regular HTML restorable.

</details>

<details>
<summary>Glossary and fonts</summary>

**Define unfamiliar terms once and choose the document's typography.**

- **Glossary:** add a `:::terms` block. The first prose occurrence in each section becomes a dotted word with a definition card, opened by click, tap, or keyboard. Cards support inline Markdown and work offline in both themes. See [glossary syntax](skill/notion-page/references/writing.md#glossary-terms).

- **Fonts:** the default `daily-dose` style uses Noto Serif for text and Domine for headings, inspired by Daily Dose of Data Science. Fonts are embedded for offline use; code stays monospace.

- **Override:** set `font: daily-dose` or `font: system` in frontmatter, or run the command below. `--font` takes precedence; `system` restores the previous system font appearance.

```sh
notion-page notes.md --font system
```

</details>

<details>
<summary>Archive and restore Markdown</summary>

**Archive mode embeds the exact Markdown in HTML and removes the source `.md` after saving.**

```sh
notion-page --archive notes.md
notion-page --restore notes.html
```

- Both commands refuse to overwrite existing output. If `notes.html` exists, use `notion-page --archive notes.md --output archives`.

- Regular conversion also refuses to overwrite an HTML archive.

- Only HTML created with `--archive` supports `--restore`. Anyone with the archive can extract its source Markdown.

</details>

<details>
<summary>CLI help and embedded guides</summary>

**Read guides from any directory, even outside the source checkout.**

```sh
notion-page --help
notion-page --version
notion-page --skill                  # List guides with descriptions
notion-page --skill simple           # Read a skill by name
notion-page --skill turkish-writing
notion-page --skill verified-notes
notion-page --skill verified-notes-reviewer
notion-page --skill writing          # Markdown writing guide
```

All installable skills above are available through `--skill <name>`.
`communication` remains an alias for `simple`. Combined skills embed the shared
communication guidance and their workflow rules directly.

</details>

<details>
<summary>Maintaining the generated skills</summary>

**Edit shared sources, regenerate skills, then build the CLI to embed the latest guides.**

1. Edit the relevant source below. Keep the short core principles in `skill/notion-page/SKILL.md` aligned with changes to `simple`.
2. Regenerate all seven outputs; do not edit generated `SKILL.md` files by hand.
3. Check freshness and run tests. Update agent installations to receive changes.

```sh
cargo run --locked --example generate-skill
cargo run --locked --example generate-skill -- --check
cargo test --locked
```

| Shared content | Edit here |
|---|---|
| Communication principles | [skill/simple/SKILL.md](skill/simple/SKILL.md) |
| Note rules | [basic-make-notes/references/rules.md](skill/basic-make-notes/references/rules.md) |
| Procedural formatting | [readable-instructions/references/rules.md](skill/readable-instructions/references/rules.md) |
| Visual checks | [visual-verification.md](skill/generate-notion-styles-with-verificate/references/visual-verification.md) |
| Source review | [reviewer.md](skill/verified-notes/references/reviewer.md) |
| Page workflow | [generate-page-workflow.md](scripts/generate-page-workflow.md) |
| Verified notes workflow | [verified-notes.template.md](scripts/verified-notes.template.md) |

- **Composition:** [`scripts/generate_skill.rs`](scripts/generate_skill.rs) combines `simple` with explicitly selected layers in `OUTPUTS`. The seven outputs are `readable-instructions`, `basic-make-notes`, `visual-verification`, `verified-notes-reviewer`, both `generate-notion-styles` variants, and `verified-notes`. Procedural rules apply only to instructions and checklists.

- **Metadata and CLI:** maintain standalone metadata in `scripts/*.template.md`; register CLI entries in [`src/cli/skills.rs`](src/cli/skills.rs). Shared sections are sorted by name, layers follow declared order, and relative shared links become GitHub links.

- **Validation:** the generator rejects duplicate source names and generated inputs, validates all inputs before writing, and leaves unchanged files alone. `--check` reports missing or stale outputs without writing.

- **Execution:** generation is an explicit, development-only Cargo example using existing Rust dependencies. It needs no Python, extra CLI, or GitHub Actions and does not run during `cargo build`.

</details>
