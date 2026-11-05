#!/usr/bin/env python3
"""Compose the installable skill from discovered skill metadata and bodies."""

import argparse
from pathlib import Path
import re
import sys
from urllib.parse import quote, urlsplit

import yaml

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = Path('skill/generate-notion-styles/SKILL.md')
TEMPLATE = Path('scripts/generate-skill.template.md')
SOURCE_URL = 'https://github.com/saadettinBerber/notion-styles-page/blob/main/'


def read_document(path, required=True):
    text = path.read_text(encoding='utf-8')
    lines = text.splitlines(keepends=True)
    if not lines or lines[0].strip() != '---':
        if required:
            raise ValueError(f'{path}: missing YAML frontmatter')
        return {}, text.strip()
    end = next((i for i in range(1, len(lines)) if lines[i].strip() == '---'), None)
    if end is None:
        raise ValueError(f'{path}: unclosed YAML frontmatter')
    data = yaml.safe_load(''.join(lines[1:end]))
    if not isinstance(data, dict):
        raise ValueError(f'{path}: frontmatter must be a mapping')
    if required:
        for key in ('name', 'description'):
            if not isinstance(data.get(key), str) or not data[key].strip():
                raise ValueError(f'{path}: {key} must be a nonempty string')
        if not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', data['name']):
            raise ValueError(f'{path}: invalid skill name')
    return data, ''.join(lines[end + 1:]).strip()


def contained(path, root):
    path = path.resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError(f'{path}: source must stay inside {root}')
    return path


def portable_body(body, source, root):
    """Resolve Markdown links for copied installations; leave code blocks intact."""
    def destination(match):
        prefix, target, suffix = match.groups()
        url = urlsplit(target)
        if url.scheme or url.netloc or target.startswith('#'):
            return match.group(0)
        path = contained(source.parent / url.path, root)
        relative = quote(path.relative_to(root.resolve()).as_posix())
        anchor = f'#{url.fragment}' if url.fragment else ''
        query = f'?{url.query}' if url.query else ''
        return prefix + SOURCE_URL + relative + query + anchor + suffix

    output = []
    fence = None
    for line in body.splitlines():
        marker = re.match(r'^\s{0,3}(`{3,}|~{3,})(.*)$', line)
        if fence:
            output.append(line)
            if marker and marker[1][0] == fence[0] and len(marker[1]) >= len(fence) and not marker[2].strip():
                fence = None
            continue
        if marker:
            fence = marker[1]
            output.append(line)
            continue
        # Preserve inline code while handling inline and reference-style links.
        parts = re.split(r'(`+[^`]*`+)', line)
        for i in range(0, len(parts), 2):
            parts[i] = re.sub(r'(\]\()([^\s()]+)(\))', destination, parts[i])
            parts[i] = re.sub(r'(^\s{0,3}\[[^]]+\]:\s*)(\S+)(.*$)', destination, parts[i])
        line = ''.join(parts)
        line = re.sub(r'^(#{1,4}) ', r'##\1 ', line)
        output.append(line)
    return '\n'.join(output)


def generate(root):
    root = root.resolve()
    skill_root = root / 'skill'
    output = root / OUTPUT
    template_data, _ = read_document(root / TEMPLATE)
    names = {template_data['name']}
    sections = []
    for path in sorted(skill_root.rglob('SKILL.md')):
        if path.is_relative_to(output.parent):
            continue
        contained(path, skill_root)
        data, body = read_document(path)
        name = data['name']
        if name in names:
            raise ValueError(f'{path}: duplicate skill name {name}')
        names.add(name)
        metadata = data.get('metadata', {})
        if not isinstance(metadata, dict):
            raise ValueError(f'{path}: metadata must be a mapping')
        source = path
        if 'source' in metadata:
            if not isinstance(metadata['source'], str) or not metadata['source'].strip():
                raise ValueError(f'{path}: metadata.source must be a nonempty path')
            source = contained(path.parent / metadata['source'], skill_root)
            if source.is_relative_to(output.parent) or source == path.resolve():
                raise ValueError(f'{path}: metadata.source cannot point to itself or the generated skill')
            source_data, body = read_document(source, required=False)
            if source_data.get('name', name) != name:
                raise ValueError(f'{path}: source skill name does not match {name}')
        if not body:
            raise ValueError(f'{source}: empty skill body')
        description = ' '.join(data['description'].split())
        relative = source.relative_to(root).as_posix()
        sections.append((name, f'## {name}\n\n{description}\n\n'
                         f'Source: [{relative}]({SOURCE_URL}{quote(relative)})\n\n'
                         + portable_body(body, source, root)))
    if not sections:
        raise ValueError('No source skills found')
    sections.sort()
    return (root / TEMPLATE).read_text(encoding='utf-8').rstrip() + '\n\n' + '\n\n---\n\n'.join(body for _, body in sections) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Fail if the generated file is missing or stale; do not write.')
    args = parser.parse_args()
    try:
        content = generate(ROOT)
        output = ROOT / OUTPUT
        current = output.read_text(encoding='utf-8') if output.exists() else None
        if content == current:
            print(f'{OUTPUT} is up to date')
            return 0
        if args.check:
            print(f'{OUTPUT} is stale; run python scripts/generate_skill.py', file=sys.stderr)
            return 1
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(content, encoding='utf-8')
        print(f'Generated {OUTPUT}')
        return 0
    except (ValueError, OSError, yaml.YAMLError) as error:
        print(error, file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
