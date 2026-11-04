# Agent guide

Follow [SKILL.md](skill/notion-sayfa/SKILL.md) when using the skill. Use this guide when maintaining the converter or skill docs.

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

Use a Python 3.10+ virtual environment for binding checks. Verify changes to `bindings/python` against the [Python API contract](skill/notion-sayfa/references/okuyucu.md) with the package installed.

```sh
cargo fmt --check --manifest-path bindings/python/Cargo.toml
PYO3_PYTHON=python cargo clippy --locked --manifest-path bindings/python/Cargo.toml --all-targets -- -D warnings
```

Record verification in [docs/development-audit.md](docs/development-audit.md).

## Showcase

- When adding a block or syntax, update the [writing guide](skill/notion-sayfa/references/yazim.md) and [showcase](skill/notion-sayfa/templates/vitrin.md) in the same commit.
- After rendering changes, build the showcase and check it in light mode, dark mode, and at phone width. If it looks different, regenerate the [README preview](docs/vitrin.png) from the output.
