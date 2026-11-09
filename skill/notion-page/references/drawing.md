# Inline SVG drawings

Place an `<svg>` and a `<figcaption>` inside `<figure>` to keep drawings within the standalone page. See the [showcase](../templates/showcase.md) for an example.

## Rules

- Keep blank lines out of `<figure>` elements. Markdown ends the HTML block at a blank line.

- Use theme classes instead of hardcoded colors:

  | Class | Purpose |
  |---|---|
  | `diagram-text` | Text |
  | `diagram-muted` | Muted text and small fills |
  | `diagram-line` | Lines |
  | `diagram-box` | Gray boxes |
  | `diagram-node` | Blue nodes |
  | `diagram-level` | Blue fills |
  | `diagram-accent` | Orange fills |
  | `diagram-accent-line` | Orange lines |

- Use a `viewBox` about 680 units wide and text at least 13 units tall. Include one SVG per figure. The converter adds horizontal scrolling, an edge fade, and a scroll hint on phones while keeping captions fixed; do not recreate these in the SVG.

- Add `role="img"` and a sentence-length `aria-label` describing the drawing.

- Verify numbers and calculations. Match decimal and percentage formatting to the document's language.

```markdown
<figure>
<svg viewBox="0 0 680 200" role="img" aria-label="A sentence describing the drawing.">
…
</svg>
<figcaption>A short explanation of what the reader is seeing.</figcaption>
</figure>
```

## Generate larger drawings with a script

For drawings longer than a few lines, use a temporary Python script and paste its output into the page. Calculate coordinates rather than maintaining them by hand.

- Keep positions in named constants such as `ROW_TOP`, `BOX_X`, and `FULL_BAR`.
- Store words and values in one dataset. Derive widths, labels, and accessibility text from it.
- Seed any randomness, for example with `random.Random(7)`, for reproducible output.
- Emit one element per line using `"\n".join(...)` to avoid blank lines.

```python
WIDTH, HEIGHT = 680, 200
OPTIONS = [("cat", 55), ("dog", 30), ("bird", 15)]
BAR_X, FULL_BAR = 120, 300

def svg():
    parts = []
    for rank, (word, share) in enumerate(OPTIONS):
        y = 40 + rank * 28
        css = "diagram-accent" if rank == 0 else "diagram-level"
        parts.append(f'<text x="30" y="{y + 12}" font-size="14" class="diagram-text">{word}</text>')
        parts.append(f'<rect x="{BAR_X}" y="{y}" width="{FULL_BAR * share / 100:.1f}" height="14" rx="4" class="{css}"/>')
    label = "Candidates: " + ", ".join(f"{w} {s} percent" for w, s in OPTIONS) + "."
    return "\n".join([f'<svg viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-label="{label}">', *parts, "</svg>"])

print(svg())
```

## Visual style

- Highlight the selected, winning, or new item in orange (`diagram-accent`, bold text). Use blue or gray for other items.
- Use small, muted text for column headings and panel captions (`diagram-muted`, 13 units).
- Separate comparison panels with a dashed vertical line (`stroke-dasharray="2 5"`); offset the right panel with `<g transform="translate(…)">`.
- Draw flow arrows with straight lines and small triangle tips. Use dashed orange lines for secondary flows such as returns.
- Number steps with small blue circles on the left.

## Review

- Route connectors around labels. Prefer right-angled paths with rounded corners (`M … L … Q …`) and put labels above horizontal segments.
- Keep punctuation attached to the preceding word, including around `<tspan>` emphasis.
- Estimate text width at roughly 7.4 units per character for 15-unit text.
- Check that percentages sum to 100 and totals and products agree.
- Keep text at least 13 units tall; allow the drawing to scroll horizontally.
- When visual verification is requested, check the drawing in a browser in both themes and at phone width.
