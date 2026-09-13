# Nebula Paste — COSMIC applet

[Français](README.md) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md)

A local **Rust/libcosmic clipboard manager** for Fedora COSMIC: text, images, links, colors, code and file references. An independent project, not affiliated with Supaste or System76.

## In preparation — 0.8

The **2A · Dust Capture** identity is integrated: [SVG preview](docs/identity-2a.svg) and [design principles](docs/DESIGN-2A.en.md). The interface retains the COSMIC theme.

The [0.8 → 1.0 roadmap](docs/ROADMAP.en.md) defines reusable templates and validation criteria. Templates are **planned, not available yet**. The binary remains 0.7.0-beta.1. This design branch builds on the 0.7 beta; the instructions below install that beta, without the new artwork on `v0.8-identity-roadmap`.

## 0.7.0-beta.1 — Search and collections

- Search image text using embedded local OCR, enabled explicitly in Preferences.
- Persistent collections: create, rename, delete and file clips by internal drag and drop. Deleting a collection keeps its clips.
- History sidebar at widths of 720 logical units or more; the panel popup supports compact and expanded modes.
- French/English UI, COSMIC theme, favorites, retention, deletion undo and manual OCR extraction retained.
- Fedora RPM build with vendored Rust dependencies and offline compilation inside rpmbuild.

This is a **beta requiring live COSMIC desktop testing**. [0.7 test guide](docs/TESTING-0.7.en.md) · [RPM build/install](packaging/README.md). Older screenshots in `docs/` depict 0.5.

## Build and install

```bash
sudo dnf install git rust cargo gcc gcc-c++ cmake make pkgconf-pkg-config libxkbcommon-devel wayland-devel fontconfig-devel freetype-devel
git clone --branch v0.7-search-collections https://github.com/nabz54/nebula-paste.git
cd nebula-paste
bash scripts/install.sh
```

Requires Rust 1.93 or newer. Cloning the public repository needs no GitHub login. Remove the previous applet from the panel before updating, then add **Nebula Paste** in COSMIC panel settings. Existing history and preferences are retained; 0.6 categories become collections on first startup.

Tesseract, Leptonica and French/English models are embedded in the executable. No system Tesseract package or runtime model download is needed. The first build can take a while.

## Usage

Use **Expand / Collapse** in the popup to switch between the compact list and the wider grid without clearing search or filters. Your choice is saved. Collections remain accessible in tabs; **Separate window** opens independent history. The panel surface renders only its icon. COSMIC may reduce popup width when screen space is limited.

Open the panel applet, search/filter, then click a card to copy its original content. Open **History** for the full window. The history desktop launcher and `nebula-paste --history` require the panel applet to be running.

Enable image search in **Preferences**. Background indexing processes one image at a time using the selected OCR language. Results participate in existing search without adding text clips. Image details show recognized text for review. Disabling the option erases the index; **Reindex** retries failures. Pausing capture prevents new automatic OCR jobs from starting; an already running job may finish.

Create a collection through **Manage collections**, then drag a clip by its handle onto the sidebar collection. You can also enter a collection in clip details. Each clip belongs to at most one collection; favorites are independent. Incoming drops from external applications are not supported.

**Shortcuts:** Ctrl+F search, Alt+arrows select, Enter copy, Ctrl+1…9 copy a page entry, Ctrl+Shift+C copy plain text, Escape back/close. Copy-only mode requires pasting with Ctrl+V afterwards. Optional direct paste requires `sudo dnf install wtype` and COSMIC virtual keyboard support; it targets whichever application has focus after closing.

## Local data and limits

History and OCR index are stored in the local Nebula Paste SQLite database with private permissions. Data is not encrypted and is never sent to an OCR service. Index text is limited to 16,384 characters per image and is removed with the source clip. Favorites survive retention and history clearing. Limits: 500 clips, 128 MiB content, 16 MiB per clip, at most 128 newly managed collections (legacy categories are preserved).

Search ignores case and Latin accents (for example, “ete” finds “été”) and combines words with type, favorite and collection filters; it is not fuzzy search. OCR may be inaccurate. File clips remain references to their original paths.

## Development

```bash
cargo fmt --check
cargo test --locked
cargo build --locked
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
```

`nebula-paste --preview` uses synthetic data with capture and copying disabled. The embedded OCR engine is C/C++; the application is Rust. Native sources, models, licenses and checksums are in `vendor/`. See [VALIDATION.md](VALIDATION.md) and the [test guide](docs/TESTING-0.7.en.md) for validation limits.
