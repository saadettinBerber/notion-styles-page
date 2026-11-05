# Agent guide

Follow [SKILL.md](skill/notion-page/SKILL.md) when using the skill. Use this guide when maintaining the converter or skill docs.

## Git

- Branches use `<type>/<short-description>` (e.g. `feat/concise-notes`); never `codex/`.
- Commit subjects follow Conventional Commits: `<type>(<scope>): <summary>` (scope optional).

## Code

- CLI parsing lives in `src/cli/mod.rs` and file handling in `src/cli/build.rs`. The library prepares documents and renders HTML.
- Keep parser types in `src/markdown.rs` and `src/markdown/`. Keep page preparation free of media reads. The Rust media layer owns local media access and image validation.
- The binary embeds `assets/template.html` and `assets/notion.css`. Use CSS variables for light and dark themes.

## Verification

Run the existing tests before changing code. Run these checks after the change.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Use a Python 3.10+ virtual environment for binding checks. Verify changes to `bindings/python` against the [Python API contract](skill/notion-page/references/reader.md) with the package installed.

```sh
cargo fmt --check --manifest-path bindings/python/Cargo.toml
PYO3_PYTHON=python cargo clippy --locked --manifest-path bindings/python/Cargo.toml --all-targets -- -D warnings
```

Record verification in [docs/development-audit.md](docs/development-audit.md).

When editing source skills or `scripts/generate-skill.template.md`, regenerate
the combined skill with `cargo run --locked --example generate-skill`. Verify
freshness with `cargo run --locked --example generate-skill -- --check`.
`cargo test --locked` includes the generator tests.

## Showcase

- When adding a block or syntax, update the [writing guide](skill/notion-page/references/writing.md) and [showcase](skill/notion-page/templates/showcase.md) in the same commit.
- After rendering changes, build the showcase and check it in light mode, dark mode, and at phone width. If it looks different, regenerate the [README preview](docs/showcase.png) from the output.
