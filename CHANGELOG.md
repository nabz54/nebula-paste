# Changelog

[Français](CHANGELOG.fr.md)

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
