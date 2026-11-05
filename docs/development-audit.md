# Development verification

## 2026-11-05 · Guide discovery and standalone skills

The generation skill lists available guides with `notion-page --skill`, selects
them by name and description, and reads the relevant documents. Guide discovery
uses the CLI catalogue. `basic-make-notes` links procedural notes to
`readable-instructions`. Added short standalone entries for `basic-make-notes`,
`readable-instructions`, and `turkish-writing`, with shared Codex/Claude links.
Each entry reads its original guide through the CLI; rules remain in their
existing files. The CLI embeds `readable-instructions` using the existing
registration structure.

Verification: all 71 Rust tests, Rustfmt, and all-target Clippy with warnings
denied passed. Skill metadata, agent links, and README shell syntax passed
validation. Reinstalled the CLI and copied all four entries into temporary
Codex and Claude Code projects. Each standalone entry returned its exact source
guide; CLI discovery listed names and descriptions for all eight guides, and
every listed guide was readable outside the checkout.

## 2026-11-05 · Turkish writing guide

Added `turkish-writing` for Turkish sentence structure, natural phrasing,
meaning and certainty, technical terminology, and spelling. Adapted the earlier
Oku language/translation guidance from `just_read`, with links pinned to its
source commit; checked language structure against published grammar/research
and spelling against TDK. Both page-generation skills now route Turkish content
to the guide. The CLI embeds it and lists its description; README documents it.

Verification: all 71 baseline and updated Rust tests passed. Rustfmt, all-target
Clippy with warnings denied, both skill metadata checks, and all 12 local links
in the four edited Markdown documents passed. Metadata validation used a
temporary Python environment with PyYAML. Reinstalled the CLI and verified its
guide listing and byte-for-byte Turkish/main guide output outside the checkout,
without creating files. Extended the existing CLI discovery/document tests.

## 2026-11-05 · Readable instructions reference

Added `skill/readable-instructions.md` as a verbatim English copy of the existing
skill. Source comparison, skill metadata validation, Rustfmt, Clippy with
warnings denied, and all 71 Rust tests passed.

## 2026-11-05 · English skill guides and paths

Translated skill instructions, concise-note guidance, reference guides, and the
showcase into English. Renamed `skill/notion-sayfa` to `skill/notion-page`, with
`writing.md`, `drawing.md`, `reader.md`, and `showcase.md` replacing the Turkish
guide filenames. Updated embedded paths, agent instructions, README links, and
the preview image path. Parser field names and Rust API identifiers retain their
existing contracts.

The CLI now lists English descriptions and the canonical guide names `writing`,
`drawing`, `showcase`, and `reader`. Legacy names remain accepted as aliases.
The minimal generation skill uses `--skill writing`.

Verification: all 70 baseline tests passed; Rustfmt, Clippy with warnings denied,
and all 71 updated Rust tests passed, including legacy guide alias coverage.
Both skill manifests, local reference links, README shell syntax, and agent
links passed validation. The reinstalled CLI returned all six complete English
guides outside the checkout. Skills CLI copied the updated generation skill
for Codex and Claude Code in temporary projects. Chromium checked the showcase
offline in light/dark desktop and 390 px phone layouts, including fonts, page
width, drawings, and toggles; no script errors or page overflow were found.
Regenerated `docs/showcase.png` from the English showcase.

## 2026-11-05 · Minimal agent skill

Added a short English `generate-notion-styles` skill with shared relative links
under `.agents/skills` for Codex and `.claude/skills` for Claude Code. It reads
CLI help, concise-note guidance, and Markdown syntax from the installed CLI.
The README's collapsible “Ajanlara ekleme” section uses agent-specific
`npx skills add` commands for Codex and Claude Code, with `--global --copy`
for user-wide installation without symlinks on macOS, Linux, and Windows.
It documents the Node.js/Git prerequisites, project scope, and invocation.

Verification: skill metadata validation, both relative links, README shell
syntax, Rustfmt, Clippy with warnings denied, all 70 Rust tests, and
`git diff --check` passed. The installed CLI provided help and both guides and
converted a Markdown note outside the checkout.
The Skills CLI's own documentation and supported-agent list confirm the new
installation flags. Installed official Node.js v26.13.0 LTS (Apple Silicon),
with npm/npx 11.20.0, after verifying the archive's SHA-256. The binaries are
available through `~/.local/bin`; npm's user prefix is `~/.local`.
Skills CLI 1.7.6 discovered the skill in the checkout and installed it with
`--copy` for both Codex and Claude Code in temporary projects. Both installed
files exactly matched the source. These checks used the local source and
project scope. README shell syntax,
Rustfmt, Clippy, and all 70 tests passed after the documentation update.

## 2026-11-05 · Selectable reading typography and CLI cleanup

Three subagents reviewed the Rust, CLI, and Python boundaries. A shared
`FontPreset` catalogue owns names, validation, and the default. The page shell
resolves explicit CLI/API options, then document `font`, then `daily-dose`.
Existing Rust functions and positional Python calls remain supported, including
`Topic(body="", **vars(document.topic))`. Presentation CSS stays in assets;
Markdown preparation and media providers do not load fonts.

The default pairs Noto Serif text with Domine headings, a 20 px/1.75 reading
rhythm (18 px on phones), paragraph spacing, and light/dark text colors.
`system` retains the previous font appearance. Six licensed variable WOFF2
files are embedded only for `daily-dose`, with full OFL notices. Original URLs
and SHA-256 checksums are in `assets/fonts/sources.json`. FontTools confirmed
Turkish glyph coverage in normal/italic fonts and weights 400–700.

The obsolete `uret` binary was removed from Cargo targets, source, and the
local Cargo installation at the user's request. Its skill parity test was
removed; output-flag alias and archive restore coverage use `notion-page`.
Installation now provides only `notion-page`.

Verification: 64 Rust tests passed before changes (after rebuilding a stale
cached test path). After changes, root Rustfmt, all-target Clippy with warnings
denied, and 70 Rust tests passed. Python 3.12 installed the final wheel;
binding Rustfmt/Clippy and 7 installed-package contract tests passed. Chrome
verified both presets offline in light/dark desktop and 390 px phone layouts:
no external requests, JavaScript errors, font loading failures, or page-level
horizontal overflow. Actual platform fonts, Turkish bold/italic text, toggles,
columns, SVG, code, and footnotes were checked. Updated the writing/API guides,
showcase, and `docs/vitrin.png`; installed-command rendering worked outside the
checkout. A separate final subagent review found no actionable issues.

## 2026-11-05 · README purpose

Added a short paragraph below the tagline explaining the shared note format across AI agents and the personal preference for a Notion-style layout. `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked` (64 tests), and `git diff --check` passed.

## 2026-11-05 · Git naming

Documented change-type branch prefixes, Conventional Commit subjects, and the `codex/` prohibition. `git diff --check` passed; source code matches the version with 64 passing tests and clean Rustfmt/Clippy.

## 2026-11-05 · Concise notes and CLI skill discovery

Added `skill/basic-make-notes.md` for concise notes that preserve enough context to make sense without the conversation. The main skill links to it. `--skill`, `--skill list`, and `--skills` list six embedded guides with one-sentence descriptions; `--skill <NAME>` prints the selected document without a source checkout. Parsing stays in `src/cli/mod.rs`; guide selection and descriptions live in `src/cli/skills.rs`.

All 60 baseline and 64 updated Rust tests passed. Rustfmt, Clippy, and skill metadata validation passed. CLI tests cover complete embedded documents, all listing forms, invalid names, conflicting conversion options, no file writes, and `uret` compatibility. The installed CLI was also checked outside the checkout. Python bindings and page rendering are unchanged.

## Markdown archives

All 60 Rust tests passed. The CLI tests cover byte-for-byte restoration, failed conversions, existing outputs, damaged archives, and protection against regular conversions overwriting an archive. Rustfmt and Clippy passed for the converter and the Python bindings. A wheel installed under Python 3.12 passed the reader and page template checks. Chrome rendered the archived showcase identically in desktop light and dark modes and at 390 pixels.

## Initial public snapshot

All 54 Rust tests passed. Rustfmt and Clippy passed for the converter and the Python bindings with Python 3.12. The CLI displayed help and version information, and the showcase HTML matched the previous build byte for byte.

## 2026-11-05 · Generated combined skill

`generate-notion-styles/SKILL.md` now includes the actual writing and conversion
rules. A Python generator discovers `skill/**/SKILL.md`, reads YAML names and
descriptions, and optionally expands a guide through `metadata.source`. No skill
list needs maintenance. The output excludes itself, has deterministic ordering,
and supports a non-writing freshness check. Relative document links become
repository URLs for copied installations; code examples remain unchanged.

The GitHub Actions workflow validates PRs with read-only permissions. On branch
pushes or manual runs, a separate job regenerates from the latest branch tip and
commits only changed output, preserving the source commit's author. It does not
force-push or bypass branch rules. README documents metadata, local generation,
and installation updates.

Verification: all 71 baseline and final Rust tests passed, as did Rustfmt and
all-target Clippy with warnings denied. Ten Python tests passed for discovery,
add/update/delete behavior, metadata validation and sources, output exclusion,
idempotence, stale checks, portable links, and preserving output on input errors.
Skill metadata validation, actionlint 1.7.12, and `git diff --check` passed.
The workflow's exact commit/push shell step was also executed against a temporary
local bare remote: new and deleted skills reached the generated remote file,
only that file was committed, the author was preserved, and a repeated run made
no commit. No GitHub-hosted run or remote publication was performed in this task.

Publication note: GitHub rejected the initial push because the OAuth session
lacked `workflow` scope. At the user's request, authentication renewal was
stopped. The workflow file is retained locally and excluded from the published
commit; the generator, tests, and generated skill are included. README identifies
automation as pending publication.

## 2026-11-05 · Rust combined-skill generator

Replaced the Python generator and its tests with a development-only Rust Cargo
example, `generate-skill`. It reuses the existing dependencies; no dependency
or lockfile changes were needed. Removed the generator's PyYAML requirements
file and replaced README maintenance commands with Cargo commands. The example
is not installed alongside `notion-page`. The optional Python bindings remain
separate.

The generator retains automatic recursive discovery, metadata-source expansion,
name ordering, output exclusion, portable links, preserved code examples,
idempotent writes, and the non-writing `--check` mode. Sixteen Rust tests cover
the former Python contracts plus name ordering, URL encoding, line-ending
normalization, and symlinks that escape the source or repository roots. These
tests run with the regular `cargo test --locked` command.

Verification: all 71 Rust baseline tests and the 10 Python baseline generator
tests passed before migration. Rustfmt, all-target Clippy with warnings denied,
all 87 final Rust tests, generation, freshness checks, and `git diff --check`
passed. The new generated skill matched the old Python generator byte for byte
using the updated template. Compared with the previous checked-in output, only
the generator-path comment changed. Cargo metadata confirms that the installed
binary target remains `notion-page` alone. No Python binding or page-rendering
code changed.
