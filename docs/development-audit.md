# Development verification

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
