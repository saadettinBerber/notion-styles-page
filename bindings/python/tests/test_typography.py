"""Page typography contracts, run against the installed notionmd-rust package."""

from dataclasses import FrozenInstanceError
import unittest

from notionmd_rust import MarkdownReader, Topic, load_page_template


class NoMedia:
    def video_data_uri(self, path):
        raise AssertionError(f"Unexpected video read: {path}")

    def image_data_uri(self, path):
        raise AssertionError(f"Unexpected image read: {path}")


class TypographyTest(unittest.TestCase):
    def setUp(self):
        self.template = load_page_template()
        self.body = "<h2>Bölüm</h2><p>Türkçe: ığüşöç İĞÜŞÖÇ <em>italik</em></p>"

    def test_existing_positional_topic_and_template_calls_use_default(self):
        topic = Topic("Başlık", "Gövde", "🧠", "", "2026-11-05", ["rust"], "Özet")
        html = self.template.render(topic, self.body)

        self.assertIsNone(topic.font)
        self.assertEqual(topic.tags, ("rust",))
        self.assertIn('data-font="daily-dose"', html)
        self.assertIn("@font-face", html)
        self.assertIn("data:font/woff2;base64,", html)
        self.assertIn(self.body, html)

    def test_document_font_roundtrips_through_public_topic(self):
        for font in (None, "daily-dose", "system"):
            with self.subTest(font=font):
                field = "" if font is None else f"font: {font}\n"
                source = f"---\nbaslik: Türkçe\n{field}---\nBir paragraf.\n"
                prepared = MarkdownReader(NoMedia()).prepare(source)
                document = prepared.document()
                topic = Topic(body="", **vars(document.topic))
                rendered = prepared.render()
                html = self.template.render(topic, rendered.html)
                selected = font or "daily-dose"

                self.assertEqual(topic.font, font)
                self.assertIn(f'data-font="{selected}"', html)
                self.assertIn("<p>Bir paragraf.</p>", html)

    def test_explicit_font_overrides_document_without_mutating_it(self):
        topic = Topic("Başlık", "", font="daily-dose")
        system_html = self.template.render(topic, self.body, font="system")
        default_html = self.template.render(topic, self.body, font=None)

        self.assertEqual(topic.font, "daily-dose")
        self.assertIn('data-font="system"', system_html)
        self.assertNotIn("@font-face", system_html)
        self.assertNotIn("data:font/woff2;base64,", system_html)
        self.assertIn('data-font="daily-dose"', default_html)
        self.assertIn("@font-face", default_html)
        with self.assertRaises(FrozenInstanceError):
            topic.font = "system"

    def test_explicit_daily_dose_overrides_system_document(self):
        topic = Topic("Başlık", "", font="system")
        html = self.template.render(topic, self.body, font="daily-dose")

        self.assertIn('data-font="daily-dose"', html)
        self.assertIn("data:font/woff2;base64,", html)
        self.assertEqual(topic.font, "system")

    def test_custom_styles_keep_precedence(self):
        styles = ':root { --font: "Custom Font", serif; }'
        html = self.template.render(Topic("Başlık", ""), self.body, extra_styles=styles)

        self.assertTrue(html.split("</style>", 1)[0].rstrip().endswith(styles))

    def test_invalid_font_names_are_rejected_at_each_public_boundary(self):
        topic = Topic("Başlık", "")
        for font in ("unknown", "", "DAILY-DOSE"):
            with self.subTest(font=font):
                with self.assertRaises(ValueError):
                    Topic("Başlık", "", font=font)
                with self.assertRaises(ValueError):
                    self.template.render(topic, self.body, font=font)
                with self.assertRaises(ValueError):
                    MarkdownReader(NoMedia()).prepare(
                        f'---\nbaslik: Başlık\nfont: "{font}"\n---\n'
                    )

    def test_non_string_fonts_are_rejected(self):
        topic = Topic("Başlık", "")
        for font in (42, ["system"], {"font": "system"}):
            with self.subTest(font=font):
                with self.assertRaises(TypeError):
                    Topic("Başlık", "", font=font)
                with self.assertRaises(TypeError):
                    self.template.render(topic, self.body, font=font)


if __name__ == "__main__":
    unittest.main()
