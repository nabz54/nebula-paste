# Nebula Paste 0.6 — Fedora COSMIC testing

[Français](TESTING-0.6.md)

Version: **0.6.0-beta.1**. Automated checks do not validate compositor focus or live panel integration. Version 0.5 results do not validate this build.

## Install

From a clean repository checkout:

```bash
git fetch origin
git switch v0.6-cosmic
bash scripts/install.sh
```

Remove and re-add the panel applet to start the new binary. Log out and back in if COSMIC keeps the old process. Existing history and preferences are preserved.

```bash
cargo test --locked
cargo build --locked
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
cargo run --locked -- --render-preview /tmp/nebula-popup-light.png popup light
cargo run --locked -- --render-preview /tmp/nebula-popup-dark.png popup dark
cargo run --locked -- --render-preview /tmp/nebula-history-light.png full light
```

## Desktop checks

- Test all four panel edges, light/dark themes, custom accents and live theme changes.
- Test 100%, 150% and 200% scaling, including a small display; all controls must remain reachable.
- The popup shows five clips per page at a width of 360 logical units.
- Open full history: the popup closes and one window shows the same data.
- Close history using its close button, Escape and Alt+F4: capture must continue.
- Run `nebula-paste --history` twice: only one history window should exist.
- Run `nebula-paste --toggle` while history is open: return to the popup.
- Resize history; switch between grid and compact list in Preferences.
- Verify Ctrl+F, Alt+arrows, Ctrl+1…5 in the popup, Enter and plain-text copy.
- Copy into an editor; test keep-open and both direct paste modes with wtype. No paste should reach Nebula itself.
- Verify favorites, collections, undo, pause, OCR, retention and English/French labels.

Report Fedora/COSMIC versions, scaling, panel position, steps and observed behavior. Use non-sensitive sample content in screenshots.

The existing storage, configuration and FR/EN translation mechanism are retained. Fluent/cosmic-config migration, snippets and image OCR indexing are future work.
