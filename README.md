# Notion Styles Page

<picture>
  <source media="(prefers-reduced-motion: reduce)" srcset="docs/vitrin.png">
  <img src="docs/readme-hero.gif" alt="A Markdown file becomes a Notion-style page with a callout, toggle, checklist, and code block.">
</picture>

**Markdown → Notion-style HTML.** One file. Opens offline.

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
notion-page --skill yazim
```

`basic-make-notes` keeps notes understandable without the conversation and
concise without losing meaning.

[Markdown guide](skill/notion-sayfa/references/yazim.md) · [Python API](skill/notion-sayfa/references/okuyucu.md)

<details>
<summary>Claude Code</summary>

Link the skill from the checkout.

```sh
mkdir -p "$HOME/.claude/skills"
ln -s "$PWD/skill/notion-sayfa" "$HOME/.claude/skills/notion-page"
```

`/notion-page`

</details>
