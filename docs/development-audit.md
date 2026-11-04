# Development verification

## Markdown archives

All 60 Rust tests passed. The CLI tests cover byte-for-byte restoration, failed conversions, existing outputs, damaged archives, and protection against regular conversions overwriting an archive. Rustfmt and Clippy passed for the converter and the Python bindings. A wheel installed under Python 3.12 passed the reader and page template checks. Chrome rendered the archived showcase identically in desktop light and dark modes and at 390 pixels.

## Initial public snapshot

All 54 Rust tests passed. Rustfmt and Clippy passed for the converter and the Python bindings with Python 3.12. The CLI displayed help and version information, and the showcase HTML matched the previous build byte for byte.
