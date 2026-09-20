# Changelog

[Français](CHANGELOG.fr.md)

## 0.9.0-beta.1 — Reliability and data control

History backup/restore with explicit conflicts and transactional rollback; private diagnostics without content; retention impact preview; native COSMIC transparency, full-width history resizing and isolated keyboard shortcuts. Reproducible search/OCR benchmarks. Live COSMIC testing remains pending. [Testing guide](docs/TESTING-0.9.en.md).

## 0.8.0-beta.1 — Reusable templates

- New **Templates** section in the popup and full window: create, edit, search, file and confirm deletion.
- Create from a text clip; templates are stored separately from history and survive retention and clearing.
- `{{name}}`, `{{date}}`, `{{server}}`, `{{ip}}` or custom fields: fill once, inspect the preview, then copy the result. Date is entered manually.
- JSON import/export with the system file chooser, preview and explicit conflict policy. Export to a **new file**; existing exports are never overwritten.
- 2A identity and French/English interface following the COSMIC theme.

Beta: automated tests and renders do not replace live Fedora COSMIC testing. [0.8 testing guide](docs/TESTING-0.8.en.md) · [Roadmap](docs/ROADMAP.en.md).

## 0.7.0-beta.1 — Search and collections

- Shorter popup: five 72-unit rows, smaller previews and side actions; opaque surface and filters without horizontal scrolling.
- Search folds Latin accents, French ligatures and decomposed accents, including OCR text, while preserving original content.
- Opt-in local image-text search with a persistent index capped at 16,384 characters per image and no extra text clips.
- Sequential background OCR, language selection, progress, recognized text in details and manual reindexing.
- Index erasure on disable and source deletion; stale results rejected after deletion or settings changes.
- Persistent collections, including empty ones: creation, atomic rename and confirmed deletion preserving clips and favorites.
- Automatic 0.6 category migration, adaptive history sidebar and internal drag-to-file.
- Upstream Fedora RPM packaging with vendored Rust dependencies, license inventory and build workflow.
- French/English documentation and test guides; migration, search, data preservation and OCR lifecycle regression tests.
- Beta: interactive Fedora COSMIC validation still required. Reusable snippets deferred to a later release.

## 0.6.0-beta.1 — COSMIC integration

- Follow the system theme for surfaces, controls, text and symbolic icons.
- Keep the panel popup compact (360 logical units, five clips per page).
- Open the shared history in a separate resizable window, without a second clipboard monitor or database writer.
- Add `--history` and a desktop launcher requiring the running panel applet.
- Preserve capture when closing history; close the active view before direct paste.
- Add bilingual action tooltips, light/dark preview rendering, lifecycle regression tests and CI checks.
- Retain all 0.5 history/preferences and embedded OCR; no data migration.
- Live Fedora COSMIC testing remains required. See [test guide](docs/TESTING-0.6.en.md).

## 0.5.0 — Unreleased

### Added
- Original color and symbolic icons, slate/cyan interface, visual identity board.
- Responsive grid and compact list with shared keyboard navigation and drag handles.
- English/French UI and persistent preferences; independent OCR language selection.
- Configurable retention with an explicit Apply action and favorite protection.
- Plain-text copying and Ctrl+Shift+C, preserving text/code content.
- Temporary and indefinite capture pauses with countdown and automatic resume.
- Twelve-second undo for the latest deletion; visible copy confirmation and optional keep-open behavior.
- English project documentation and bilingual changelogs.

### Fixed
- Undo is transactional: it preserves metadata, does not overwrite a recapture and does not evict other history entries.
- Expired undo requests and clearing history cannot restore deleted data.
- Temporary pause uses a monotonic clock and invalidates earlier capture epochs.
- Plain-text conversion does not strip markup from source code.
- Preference writes use a unique private temporary file followed by atomic replacement.

### Status
Development preview on `v0.5-design`. See VALIDATION.md for tested behavior; live Fedora COSMIC validation is still required before a stable release.

## 0.4.0 — 2026-09-09
- Embedded Tesseract/Leptonica and French/English models; offline OCR without a system Tesseract executable.
- OCR command-line diagnostic and embedded-runtime verification script.
- Bundled native sources, checksums and third-party licenses.

## 0.3.0
- Outgoing native drag-and-drop, optional direct paste with wtype and image OCR.
- Richer previews and improved history navigation.

## 0.2.0
- Opaque popup surfaces, rectangular cards and improved readability.

## 0.1.0
- Initial Rust/COSMIC clipboard applet with local history, filters and favorites.

Earlier entries summarize the project's development history; they are not a claim that GitHub releases or tags exist for each version.
