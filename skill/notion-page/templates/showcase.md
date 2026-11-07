---
title: Template Showcase
ikon: 🧪
kapak: purple
tarih: 2026-10-07
etiketler: [template, example]
ozet: An example page covering every block supported by the converter.
---

This page demonstrates how the converter renders each supported block. Copy it to start a new page.

The default `daily-dose` preset uses Noto Serif body text and Domine headings.
It supports **bold text**, *italic text*, and Unicode such as café and naïve.
Fonts are embedded for offline use. Select the previous appearance with
`font: system` in frontmatter or `--font system` on the command line.

:::callout 💡 blue
A **callout** highlights a main idea. It supports **bold**, *italic*, `code`, and [links](https://example.com).
:::

# Heading 1

A plain paragraph with ==highlighted text== and ~~strikethrough~~.

Select any passage while reading and choose **Vurgula** to add your own yellow
highlight. Select it again and choose **Vurguyu kaldır** to erase that portion.
Choose **Vurgulu HTML’i kaydet** to download a copy with your reading marks.
Open that copy in another browser or share it with someone else; the marks stay
in the HTML. Save again after edits. Reading marks never change Markdown or the
source recovered by `--restore`.

## Heading 2

- A bullet list
- A second item
  - A nested item

1. A numbered list
2. A second step

### Heading 3

- [x] Completed task
- [ ] Pending task

> A blockquote with a thick left border, as in Notion.

:::callout ⚠️ yellow
A yellow callout for a warning.
:::

:::callout ✅ green
A green callout for a positive result.
:::

:::toggle Collapsible section (click)
Toggle content starts hidden. Use it for details, extra explanations, or optional notes.

- Lists work inside toggles too.
:::

:::toggle-heading Prominent toggle heading
This uses the same syntax as a toggle with an h1-sized title. Use it sparingly for a prominent section or closing thought.
:::

::::callout 🧭 purple
Outer containers use more colons than inner containers.

:::toggle Toggle inside a callout
Use `::::` outside and `:::` inside.
:::
::::

::::columns
:::column
**Left column**

Two columns help compare ideas.
:::
:::column
**Right column**

Columns stack vertically on phones.
:::
::::

---

<figure>
<svg viewBox="0 0 680 140" role="img" aria-label="From left to right: notion-page converts notes.md into a standalone page.html file.">
<rect x="40" y="40" width="160" height="56" rx="10" class="diagram-box" stroke-width="1.5"/>
<text x="120" y="74" text-anchor="middle" font-size="15" class="diagram-text">notes.md</text>
<line x1="200" y1="68" x2="250" y2="68" class="diagram-line" stroke-width="2"/>
<polygon points="250,62 260,68 250,74" class="diagram-muted"/>
<rect x="260" y="40" width="160" height="56" rx="10" class="diagram-node" stroke-width="1.5"/>
<text x="340" y="74" text-anchor="middle" font-size="15" font-weight="600" class="diagram-text">notion-page</text>
<line x1="420" y1="68" x2="470" y2="68" class="diagram-line" stroke-width="2"/>
<polygon points="470,62 480,68 470,74" class="diagram-muted"/>
<rect x="480" y="40" width="160" height="56" rx="10" class="diagram-accent-line" stroke-width="2"/>
<text x="560" y="74" text-anchor="middle" font-size="15" font-weight="600" class="diagram-accent">page.html</text>
<text x="120" y="124" text-anchor="middle" font-size="13" class="diagram-muted">your Markdown</text>
<text x="340" y="124" text-anchor="middle" font-size="13" class="diagram-muted">convert</text>
<text x="560" y="124" text-anchor="middle" font-size="13" class="diagram-muted">one file, offline</text>
</svg>
<figcaption>An inline SVG drawing. Theme classes keep it readable in dark mode.</figcaption>
</figure>

A video block requires a media file, so only its syntax is shown here:

```markdown
:::video media/demo.mp4 media/demo-poster.jpg
Video caption.
:::
```

The Markdown image syntax is shown below. Library callers can opt into embedding
PNG, JPEG, GIF, and WebP files; regular `notion-page` conversion preserves their
paths. This example stays in a code fence so the showcase needs no extra files:

```markdown
![Image description](media/image.png)
```

| Concept | Description |
|---|---|
| Row 1 | Tables use thin borders, as in Notion |
| Row 2 | Header rows have a subtle gray background |

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

Blocks tagged `text` wrap long lines inside the box. Use them for prompts or logs:

```text
This intentionally long line demonstrates wrapping: code blocks normally scroll sideways, but a block tagged text wraps at the right edge and remains within the page width.
```

Container markers inside code fences remain literal text, including unknown names:

```text
:::example
These lines do not open a container.
:::
```

## Glossary terms

BREX checks a project's S1000D rules, while XSD describes an XML document's
structure. A crate packages reusable Rust code. Click a dotted word to read its
definition, or reach it with Tab and press Enter. Press Esc to close the card.

BREX appears again here without another underline because only its first use in
this section is marked.

Footnotes are supported too.[^1]

[^1]: Footnotes are collected at the bottom of the page.

:::terms
- **BREX:** Business Rules Exchange: a project's own **S1000D rules**.
- **XSD:** XML Schema Definition: a schema for the structure of an XML document.
- **Crate:** A Rust library package, such as `serde`. See the [Rust guide](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html).
:::
