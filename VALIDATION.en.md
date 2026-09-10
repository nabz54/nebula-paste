# Validation — 0.5.0-dev.1, 10 September 2026

- Development build completed on Linux with Rust 1.98.1 and locked dependencies.
- **33 tests passed**: 4 library/settings/language tests, 17 app/OCR/transfer tests, and 12 history/integrity/retention/undo tests.
- `cargo fmt --check` and Bash syntax checks passed.
- Real English, French and combined OCR passed with an empty PATH, system models ignored, and no dynamic Tesseract/Leptonica dependency.
- Actual widget screenshots rendered and inspected: French/English 940 px grid, 470 px compact list and preferences, and 360 px narrow grid. Header and pagination clipping were fixed after inspection.

This was a development build, not a release build. No live Fedora COSMIC session was available. Popup placement, compositor resizing, Wayland transfers, focus return, symbolic icon theming and drag-and-drop still require testing on that desktop. Headless screenshots do not validate those interactions.

The supplied patch was integrated with transactional undo, lossless plain-text handling, monotonic pause timing, explicit retention application and unique temporary preference files.
