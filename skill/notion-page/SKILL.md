---
name: notion-page
description: Convert Markdown notes into a standalone, offline Notion-style HTML page using the notion-page CLI.
---

**Turn the user's content into a Notion-style page.**

- Read `notion-page --help`. If the CLI is missing, follow the repository's [installation instructions](../../README.md#install).

- Apply [basic-make-notes](../basic-make-notes.md) when writing or shortening notes. Preserve the user's meaning, examples, and order; clarify missing information that affects the result.

- When writing in Turkish, also follow the [Turkish writing guide](references/turkish-writing.md) for natural phrasing, terminology, and spelling.

- Start Markdown with YAML frontmatter containing `title`. Follow the [writing guide](references/writing.md); consult the [showcase](templates/showcase.md) for supported blocks and the [drawing guide](references/drawing.md) for inline SVG.

- Run `notion-page notes.md`, or `notion-page notes/*.md --output pages`. Fix any reported errors before sharing the output.

- Check the HTML in light and dark themes and at phone width. Look for overflow, overlapping labels, unclosed `:::` markers, and unreadable colors. If a browser cannot open a large embedded video through `file://`, serve the output with `python3 -m http.server 8000`.

## CLI guides

---

**Read embedded guides without a source checkout.**

`notion-page --skill` lists guides. Use `notion-page --skill NAME` to read `notion-page`, `basic-make-notes`, `readable-instructions`, `turkish-writing`, `writing`, `drawing`, `showcase`, or `reader`.

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
