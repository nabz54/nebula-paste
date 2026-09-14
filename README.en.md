# Nebula Paste — COSMIC applet

[Français](README.md) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md)

A local **Rust/libcosmic clipboard manager** for Fedora COSMIC: text, images, links, colors, code and file references. An independent project, not affiliated with Supaste or System76.

## 0.8.0-beta.1 — Reusable templates

- New **Templates** section in the popup and full window: create, edit, search, file and confirm deletion.
- Create from a text clip; templates are stored separately from history and survive retention and clearing.
- `{{name}}`, `{{date}}`, `{{server}}`, `{{ip}}` or custom fields: fill once, inspect the preview, then copy the result. Date is entered manually.
- JSON import/export with the system file chooser, preview and explicit conflict policy. Export to a **new file**; existing exports are never overwritten.
- 2A identity and French/English interface following the COSMIC theme.

Beta: automated tests and renders do not replace live Fedora COSMIC testing. [0.8 testing guide](docs/TESTING-0.8.en.md) · [Roadmap](docs/ROADMAP.en.md).

## Build and install

```bash
sudo dnf install git rust cargo gcc gcc-c++ cmake make pkgconf-pkg-config libxkbcommon-devel wayland-devel fontconfig-devel freetype-devel
git clone --branch main https://github.com/nabz54/nebula-paste.git
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
