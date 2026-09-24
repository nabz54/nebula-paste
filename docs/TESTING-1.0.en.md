# 1.0 acceptance testing

[Français](TESTING-1.0.md)

1.0.0-rc.1 prepares the stable release. Automated success is not evidence of real COSMIC behavior.

## Observed status

- PR #8 popup blur fix: tests, compilation and RPM build passed for commit `3aec578b0da85ebb221ff6fdad95fc18f01a20f1`; positive user feedback on September 24, 2026 after installing the corrected package.
- This feedback does not validate every monitor, theme, separate window or restoration workflow.
- Check this candidate's own commit runs. PR #8 results do not establish candidate validation.

## Stable release acceptance

Record the date, `rpm -q nebula-paste cosmic-comp cosmic-panel`, session type and display scale factors. No personal clipboard content is needed.

| Scenario | Acceptance criterion | Status |
|---|---|---|
| Fresh RC installation | Panel integration, capture, login restart | Pending |
| 0.9 → RC upgrade | History, favorites, collections, templates and preferences retained | Pending |
| Local-install migration | One `/usr/bin/nebula-paste` process and correct launchers | Pending on RC |
| Compact/expanded popup | Calendar-like blur, retained search/filters, reopen | Fix confirmed by user; recheck RC |
| Separate history window | Resize, close, theme and transparency | Pending |
| Keyboard and clipboard | Search, navigation, text/image copy, Ctrl+V and optional direct paste | Pending |
| Themes and displays | Light/dark, blur on/off, relevant panel positions and scales | Pending |
| Backup/restore | Separate history/template exports, preview, conflicts, restart and cancellation | Pending |
| OCR | French/English, disable and reindex | Pending |
| Sustained usage | Several weeks without data loss, crashes or known blocking defects | To document |

Follow the detailed [0.8](TESTING-0.8.en.md) and [0.9](TESTING-0.9.en.md) test guides. Use synthetic clips for import and retention testing.

## Automation

`cargo fmt --check`, `cargo test --locked`, `cargo build --locked`, desktop validation and `scripts/check-embedded-ocr.sh`. Release branches build the Fedora 44 RPM after tests; packaging also checks bundled OCR. Main-branch publication depends on tests and packaging and attaches RPMs and checksums to the same release tag.

## Stable decision

Close blocking defects, record the scenarios above and remaining limitations, then update Cargo/RPM versions and changelogs together to 1.0.0. There is no automatic promotion to stable. Packages remain unsigned; support for other Fedora versions is not promised.
