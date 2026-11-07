# Source-preserving document reader

The Python binding requires Cargo and Python 3.10+. Install it from the repository root:

```sh
python -m pip install ./bindings/python
```

The Rust entry point is `notion_sayfa::prepare_document`; the Python entry point is `notionmd_rust.MarkdownReader`.

```python
from pathlib import Path

from notionmd_rust import MarkdownReader, MediaFolder, Topic, load_page_template

path = Path("notes.md")
source = path.read_bytes().decode("utf-8")
media = MediaFolder(path.parent)
prepared = MarkdownReader(media).prepare(source, id_prefix="en-")
document = prepared.document()
# Apply media and content policies using document here.
rendered = prepared.render()
if document.topic is not None:
    topic = Topic(body="", **vars(document.topic))
    page_html = load_page_template().render(topic, rendered.html, lang="en")
```

`prepare()` parses a page with frontmatter or a headerless Markdown fragment. `document()` exposes source ranges and queries. Neither step reads media. `render()` produces body HTML and a render tree from the prepared document. An optional `media` argument allows rendering the same document with another provider, which supplies `video_data_uri(path)` and `image_data_uri(path)`.

## Source coordinates and queries

Range `start` and `end` values are Unicode code-point indices into the original source, with an exclusive end. `source[span.start:span.end]` returns the exact slice. Read bytes as in the example to preserve BOM and CRLF.

- `topic` contains frontmatter metadata; `header_fields` contains field names and original value ranges. Headerless fragments have `topic=None`.
- `blocks` contains kind, range, shape signature, shareability, and children. Toggle headings expose `title` and `title_range`.
- `image_destinations` contains resolved image targets and ranges; `container_uses` contains Notion containers and ranges.
- `glossary_terms` contains each `:::terms` entry's `term`, original inline Markdown `definition`, and `source_range` for its list item. It is empty when no glossary is present. The block appears in `container_uses` as `terms` and in `blocks` as `container:terms`, with `glossary_term` children.
- `protected_ranges` identifies code, link targets, and similar protected spans. `plain_text` is the text query.
- `formatting` contains `content_text`, `code_signatures` with original code bytes, `raw_html_ranges`, and `container_titles`.

Invalid YAML and unknown or unclosed containers raise Python `ValueError`. Container errors include the original line number. Malformed glossary entries, duplicate terms, and multiple terms blocks also raise `ValueError` with the original source line. Preparation validates the glossary without reading media.

## HTML and media

`rendered.html` is the HTML body. `rendered.render_nodes` exposes the render tree for operations such as bilingual layouts. Leaves provide `html`, `shared`, and `inline`; branches provide opening and closing HTML and children. Footnote branches also include identity and order data.

`id_prefix` separates generated heading, footnote, and glossary IDs for documents embedded in one page. For example, `en-` produces `fn-en--1` for the first footnote and `en-term-brex` for the BREX card. Glossary slugs are ASCII with Turkish letters folded and avoid heading ID collisions. `MarkdownReader(media, allow_raw_html=False)` escapes raw HTML as text. Source coordinates do not depend on rendering options.

The terms block emits no HTML at its source position. Marked words link to native popover cards appended at the end of `rendered.html`; the cards are also represented in `render_nodes`, so assembling the tree reproduces the body. The [writing guide](writing.md#glossary-terms) describes section boundaries, suffix matching, and excluded content. Passing the body to `load_page_template().render()` includes the card styles and positioning script only when needed.

When a term has no eligible occurrence, `rendered.warnings` contains its warning. The field is omitted from the Python result when there are no warnings. Library rendering does not write to stderr; CLI conversion reports warnings and still writes the page. Rust page callers can use `render_page_with_diagnostics(page, media, options)` to receive `RenderedPage { html, warnings }`; existing `render_page` and `render_page_with_options` continue to return HTML strings.

Opt into embedding body images with `prepared.render(embed_images=media.embedded_image_data_uri)`. The callback receives the resolved destination, which is preserved in `data-source`. The Rust media layer validates regular PNG, JPEG, GIF, and WebP bytes, including every animation frame. `MediaFolder` rejects symlinks, absolute paths, and parent traversal. `inspect_image(content)` performs the same validation without reading files and returns `ImageInfo(media_type, width, height)`. The caller decides whether media access is permitted.

## Page shell and typography

`load_page_template().render(topic, body_html, lang="tr", extra_styles="", font=None)` places the body into the page shell. Pass `lang="en"` for English output. `font` accepts `"daily-dose"` or `"system"`. The default `daily-dose` embeds Noto Serif body text and Domine headings for offline use. `system` retains operating-system fonts and the previous page dimensions. Code fonts are independent. `extra_styles` is applied last.

Frontmatter `font` is exposed as `document.topic.font`, or `None` when omitted. The final optional `Topic` constructor argument is also `font=None`, so `Topic(body="", **vars(document.topic))` preserves the setting. Like other topic fields, `font` is immutable. An explicit template font overrides the topic font; if both are `None`, `daily-dose` is used. Unknown font names raise `ValueError`; non-string Python values raise `TypeError`.

```python
topic = Topic("Notes", "", font="system")
page_html = load_page_template().render(topic, rendered.html, font="daily-dose")
```

In Rust, pass `PageOptions { font: Some(FontPreset::System) }` to `render_page_with_options` or `render_shell_with_options`. Existing `render_page` and `render_shell` functions use the document preference and shared default. Typography options do not change the Markdown body, document queries, or media provider.

`writing_guide()` returns the embedded [canonical writing guide](writing.md).
