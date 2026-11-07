"""Glossary contracts, run against the installed notionmd-rust package."""

import unittest
from html.parser import HTMLParser

from notionmd_rust import MarkdownReader, Topic, load_page_template


class NoMedia:
    def video_data_uri(self, path):
        raise AssertionError(f"Unexpected video read: {path}")

    def image_data_uri(self, path):
        raise AssertionError(f"Unexpected image read: {path}")


def assemble(nodes):
    return "".join(
        node.html
        if hasattr(node, "html")
        else node.opening_html + assemble(node.children) + node.closing_html
        for node in nodes
    )


class TermButtons(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.buttons = []
        self.current = None

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == "button" and attrs.get("class") == "term":
            self.current = [attrs["popovertarget"], ""]

    def handle_data(self, data):
        if self.current is not None:
            self.current[1] += data

    def handle_endtag(self, tag):
        if tag == "button" and self.current is not None:
            self.buttons.append(tuple(self.current))
            self.current = None


class TermsTest(unittest.TestCase):
    def setUp(self):
        self.reader = MarkdownReader(NoMedia())
        self.source = (
            "\ufeff---\r\n"
            "title: Türkçe 🧠\r\n"
            "---\r\n"
            "BREX'e bakın.\r\n\r\n"
            ":::terms\r\n"
            "- **BREX:** Projenin **kuralları**, `XML` ve [bağ](https://example.test).\r\n"
            "- **Çağrı:** Bir işlevin çalıştırılması.\r\n"
            ":::\r\n"
        )

    def test_prepare_exposes_terms_and_original_unicode_ranges_without_media(self):
        prepared = self.reader.prepare(self.source)
        document = prepared.document()

        self.assertEqual(document.source, self.source)
        self.assertEqual(document.html, "")
        self.assertEqual(document.render_nodes, ())
        self.assertEqual([item.term for item in document.glossary_terms], ["BREX", "Çağrı"])
        first = document.glossary_terms[0]
        self.assertEqual(
            first.definition,
            "Projenin **kuralları**, `XML` ve [bağ](https://example.test).",
        )
        self.assertEqual(
            self.source[first.source_range.start : first.source_range.end].rstrip("\r\n"),
            "- **BREX:** " + first.definition,
        )
        container = next(item for item in document.container_uses if item.name == "terms")
        self.assertEqual(
            self.source[container.source_range.start : container.source_range.end],
            self.source[self.source.index(":::terms") :],
        )
        block = next(item for item in document.blocks if item.kind == "container:terms")
        self.assertEqual([item.kind for item in block.children], ["glossary_term"] * 2)
        self.assertEqual(block.html, "")

    def test_render_uses_prefixed_ids_inline_definitions_and_complete_render_tree(self):
        rendered = self.reader.prepare(self.source, id_prefix="en-").render()

        self.assertIn('popovertarget="en-term-brex"', rendered.html)
        self.assertIn('id="en-term-brex" class="term-card" popover', rendered.html)
        self.assertIn('id="en-term-cagri" class="term-card" popover', rendered.html)
        self.assertIn("<strong>kuralları</strong>", rendered.html)
        self.assertIn("<code>XML</code>", rendered.html)
        self.assertIn('<a href="https://example.test">bağ</a>', rendered.html)
        self.assertNotIn(":::terms", rendered.html)
        self.assertEqual(assemble(rendered.render_nodes), rendered.html)
        self.assertEqual([item.term for item in rendered.glossary_terms], ["BREX", "Çağrı"])
        self.assertEqual(len(rendered.warnings), 1)
        self.assertIn("Çağrı", rendered.warnings[0])

        unprefixed = self.reader.prepare(self.source).render()
        self.assertNotIn("en-term-", unprefixed.html)

    def test_page_template_adds_glossary_assets_only_when_cards_exist(self):
        template = load_page_template()
        topic = Topic("Sözlük", "", font="system")
        rendered = self.reader.prepare(self.source).render()
        page = template.render(topic, rendered.html)
        plain = template.render(topic, "<p>Bir paragraf.</p>")

        self.assertIn(".term-card", page)
        self.assertIn("<!-- terms-script:start -->", page)
        self.assertNotIn(".term-card", plain)
        self.assertNotIn("<!-- terms-script:start -->", plain)
        for html in (page, plain):
            self.assertIn('id="reader-highlight-data"', html)

    def test_formatted_suffixes_and_multiword_terms_preserve_the_render_tree(self):
        source = (
            "**BREX**'e ve veri **modülünü** inceleyin.\n\n"
            "## İkinci bölüm\n\n"
            "BREX&#39;e ve **veri** *modülünü* inceleyin.\n\n"
            ":::terms\n"
            "- **BREX:** Proje kuralları.\n"
            "- **Veri modülü:** Yeniden kullanılabilir bilgi birimi.\n"
            ":::\n"
        )
        rendered = self.reader.prepare(source, id_prefix="tr-").render()
        buttons = TermButtons()
        buttons.feed(rendered.html)

        self.assertEqual(
            buttons.buttons,
            [
                ("tr-term-brex", "BREX'e"),
                ("tr-term-veri-modulu", "veri modülünü"),
            ]
            * 2,
        )
        self.assertEqual(assemble(rendered.render_nodes), rendered.html)
        self.assertFalse(hasattr(rendered, "warnings"))

    def test_invalid_terms_report_original_source_lines(self):
        cases = [
            (":::terms\r\n- broken\r\n:::\r\n", "- broken"),
            (
                ":::terms\r\n- **BREX:** One.\r\n- **BREX:** Two.\r\n:::\r\n",
                "- **BREX:** Two.",
            ),
            (
                ":::terms\r\n- **BREX:** One.\r\n:::\r\n\r\n"
                ":::terms\r\n- **XSD:** Two.\r\n:::\r\n",
                ":::terms\r\n- **XSD:",
            ),
        ]
        prefix = "\ufeff---\r\ntitle: Başlık\r\n---\r\nParagraf.\r\n\r\n"
        for body, invalid in cases:
            source = prefix + body
            line = source[: source.index(invalid)].count("\n") + 1
            with self.subTest(body=body):
                with self.assertRaisesRegex(ValueError, f"Satır {line}:"):
                    self.reader.prepare(source)

    def test_headerless_documents_and_missing_glossaries_remain_supported(self):
        source = "Crate\n\n:::terms\n- **Crate:** A package.\n:::\n"
        rendered = self.reader.prepare(source).render()

        self.assertIsNone(rendered.topic)
        self.assertIn('popovertarget="term-crate"', rendered.html)
        self.assertFalse(hasattr(rendered, "warnings"))

        plain = self.reader.prepare("A paragraph.\n").render()
        self.assertEqual(plain.glossary_terms, [])
        self.assertEqual(plain.html, "<p>A paragraph.</p>\n")
        self.assertEqual(assemble(plain.render_nodes), plain.html)


if __name__ == "__main__":
    unittest.main()
