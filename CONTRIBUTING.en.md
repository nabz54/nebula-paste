# Contributing

Read [README.en.md](README.en.md) for build dependencies and installation. Work on a focused branch and explain the problem, resulting behavior and checks performed in your pull request.

```bash
cargo build --locked
cargo fmt --check
cargo test --locked
bash -n scripts/install.sh scripts/uninstall.sh scripts/check-embedded-ocr.sh
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
```

For interface changes, render and inspect previews in both languages and test in a real COSMIC session. Use synthetic clipboard data in tests and screenshots. Keep Cargo.lock committed. Changes to embedded OCR sources or models must update vendor checksums and relevant licenses.

UI translations use the `tr!` and `tr_format!` macros. Never translate clipboard content or identifiers. Preserve French and English strings when adding controls.

Project source is MPL-2.0; third-party licenses are retained in vendor/licenses/.
