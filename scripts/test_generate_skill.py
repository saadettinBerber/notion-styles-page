import shutil
import subprocess
import sys
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from generate_skill import OUTPUT, ROOT, TEMPLATE, generate, read_document


class GenerateSkillTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'scripts').mkdir()
        shutil.copy(ROOT / TEMPLATE, self.root / TEMPLATE)
        shutil.copy(ROOT / 'scripts/generate_skill.py', self.root / 'scripts/generate_skill.py')

    def skill(self, folder='example', name='example', description='Use for examples.', body='Keep the context.', extra=''):
        path = self.root / 'skill' / folder / 'SKILL.md'
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f'---\nname: {name}\ndescription: {description}\n{extra}---\n\n{body}\n')
        return path

    def cli(self, *args):
        return subprocess.run([sys.executable, str(self.root / 'scripts/generate_skill.py'), *args], capture_output=True, text=True)

    def test_add_update_delete_are_discovered_without_configuration(self):
        self.skill()
        before = generate(self.root)
        path = self.skill('nested/new-skill', 'new-skill', body='New guidance.')
        self.assertIn('New guidance.', generate(self.root))
        path.write_text(path.read_text().replace('New guidance.', 'Changed guidance.'))
        self.assertIn('Changed guidance.', generate(self.root))
        self.assertNotIn('New guidance.', generate(self.root))
        path.unlink()
        self.assertEqual(before, generate(self.root))

    def test_output_excluded_and_reruns_are_identical(self):
        self.skill()
        self.assertEqual(0, self.cli().returncode)
        output = self.root / OUTPUT
        before = output.read_bytes()
        modified = output.stat().st_mtime_ns
        self.assertEqual(0, self.cli().returncode)
        self.assertEqual(before, output.read_bytes())
        self.assertEqual(modified, output.stat().st_mtime_ns)
        self.assertEqual(0, self.cli('--check').returncode)
        self.assertEqual('generate-notion-styles', read_document(output)[0]['name'])

    def test_check_reports_stale_without_writing(self):
        path = self.skill()
        self.assertEqual(1, self.cli('--check').returncode)
        self.assertFalse((self.root / OUTPUT).exists())
        self.assertEqual(0, self.cli().returncode)
        previous = (self.root / OUTPUT).read_bytes()
        path.write_text(path.read_text() + '\nAn added rule.\n')
        self.assertEqual(1, self.cli('--check').returncode)
        self.assertEqual(previous, (self.root / OUTPUT).read_bytes())

    def test_metadata_source_includes_rules_and_tracks_edits(self):
        self.skill(body='Read the guide through the CLI.', extra='metadata:\n  source: ../guide.md\n')
        guide = self.root / 'skill/guide.md'
        guide.write_text('---\nname: example\n---\nActual rules.')
        self.assertIn('Actual rules.', generate(self.root))
        self.assertNotIn('Read the guide through the CLI.', generate(self.root))
        guide.write_text('Updated rules without frontmatter.')
        self.assertIn('Updated rules without frontmatter.', generate(self.root))
        guide.unlink()
        with self.assertRaises(FileNotFoundError):
            generate(self.root)

    def test_yaml_multiline_description_and_unicode_body(self):
        self.skill(description='>\n  Apply to instructions:\n  preserve their meaning.', body='Türkçe açıklamayı koru.')
        result = generate(self.root)
        self.assertIn('Apply to instructions: preserve their meaning.', result)
        self.assertIn('Türkçe açıklamayı koru.', result)

    def test_duplicate_missing_or_invalid_metadata_fail(self):
        path = self.skill()
        duplicate = self.skill('duplicate')
        with self.assertRaisesRegex(ValueError, 'duplicate skill name'):
            generate(self.root)
        duplicate.unlink()
        for text in ('Missing frontmatter', '---\nname: example\n---\nRule.', '---\nname: []\ndescription: Example\n---\nRule.', '---\nname: example\ndescription: [oops\n---\nRule.'):
            path.write_text(text)
            self.assertNotEqual(0, self.cli().returncode)
            self.assertFalse((self.root / OUTPUT).exists())

    def test_source_cannot_escape_or_include_generated_output(self):
        for source in ('../../outside.md', '../generate-notion-styles/SKILL.md', 'SKILL.md'):
            self.skill(extra=f'metadata:\n  source: {source}\n')
            with self.assertRaises(ValueError):
                generate(self.root)

    def test_relative_links_are_portable_and_code_is_unchanged(self):
        self.skill(body='''# Rules
[Other](../other.md#notes)
[Online](https://example.com/a)
[Ref]: ../other.md
`[Code](../other.md)`
```markdown
# Heading example
[Code](../other.md)
```
''')
        result = generate(self.root)
        self.assertIn('/blob/main/skill/other.md#notes)', result)
        self.assertIn('[Ref]: https://github.com/', result)
        self.assertIn('[Online](https://example.com/a)', result)
        self.assertIn('`[Code](../other.md)`', result)
        self.assertIn('```markdown\n# Heading example\n[Code](../other.md)\n```', result)
        self.assertIn('### Rules', result)

    def test_invalid_source_does_not_replace_existing_output(self):
        path = self.skill()
        self.assertEqual(0, self.cli().returncode)
        before = (self.root / OUTPUT).read_bytes()
        path.write_text('Invalid metadata')
        self.assertEqual(1, self.cli().returncode)
        self.assertEqual(before, (self.root / OUTPUT).read_bytes())

    def test_source_name_must_match_discoverable_skill(self):
        self.skill(extra='metadata:\n  source: ../guide.md\n')
        (self.root / 'skill/guide.md').write_text('---\nname: wrong-guide\n---\nRules.')
        with self.assertRaisesRegex(ValueError, 'source skill name does not match'):
            generate(self.root)


if __name__ == '__main__':
    unittest.main()
