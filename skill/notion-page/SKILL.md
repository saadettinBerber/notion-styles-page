---
name: notion-page
description: Convert Markdown notes into a standalone, offline Notion-style HTML page using the notion-page CLI.
---

**Turn the user's content into a Notion-style page.**

- Read `notion-page --help`. If the CLI is missing, follow the repository's [installation instructions](https://github.com/saadettinBerber/notion-styles-page#install).

- To update an existing installation, run `notion-page update --source /path/to/notion-styles-page`. This rebuilds the CLI and refreshes installed skills from that source. Add `--pull` to fetch the checkout's upstream changes first. From the source directory, `npx --yes --package=. notion-page update` also works with Rust/Cargo and Node.js/npm installed.

- **Be concise: use fewer words without losing meaning.** In user-facing replies, progress updates, and documents, lead with what the reader needs. Explain the reason and useful detail in natural language. Use shared context in conversation and enough context for the intended reader in standalone notes. Preserve meaning, examples, conditions, and uncertainty. Choose paragraphs, lists, or tables to fit the content. Report the result and the scope of verification. For additional examples in a standalone installation, read `notion-page --skill simple`.

- When writing in Turkish, apply the Turkish writing guidance included in the combined skill. For a standalone page conversion, read `notion-page --skill turkish-writing` for natural phrasing, terminology, and spelling.

- Start Markdown with YAML frontmatter containing `title`. Follow the [writing guide](references/writing.md); consult the [showcase](templates/showcase.md) for supported blocks and the [drawing guide](references/drawing.md) for inline SVG.

- Define unfamiliar vocabulary once in a `:::terms` block using `- **Term:** definition`. Each section's first prose occurrence opens a definition card. See [glossary terms](references/writing.md#glossary-terms) for matching rules and excluded content.

- Run `notion-page notes.md`, or `notion-page notes/*.md --output pages`. Fix any reported errors before sharing the output.

## CLI guides

---

**Read embedded guides without a source checkout.**

`notion-page --skill` lists guides. Use `notion-page --skill NAME` to read `notion-page`, `simple`, `turkish-writing`, `writing`, `drawing`, `showcase`, or `reader`. The communication principles are included above and in the combined skill.

## Output options

---

**Choose output options to match the request.**

- Typography defaults to embedded Noto Serif body text and Domine headings (`daily-dose`). Use `--font system` for system fonts. CLI `--font` overrides frontmatter `font`; code keeps its monospace font.

- When the user wants to keep only HTML, use `notion-page --archive notes.md`. It embeds the Markdown and deletes the source after saving. `notion-page --restore notes.html` restores the exact source. Choose another `--output` directory if the destination exists.

## Common mistakes

---

**Keep the document valid and shareable.**

- Use `##` for body sections instead of repeating the frontmatter title.

- Give outer containers more colons than inner ones (`:::::` > `::::` > `:::`).

- Keep blank lines out of `<figure>` elements. Use supported color names such as `blue`, `yellow`, or `green`.

- Videos and inline SVG are embedded. Cover and Markdown images remain linked by default; use reachable HTTPS URLs when sharing a single file. See [media handling](references/writing.md#media).
