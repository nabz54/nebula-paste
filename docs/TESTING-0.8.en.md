# Testing Nebula Paste 0.8 beta

[Français](TESTING-0.8.md)

## Installation and data

Remove the applet from the panel, install this version, then add it again. Check `nebula-paste --version`: `0.8.0-beta.1`. Existing 0.7 history, favorites, collections and OCR index must remain. An additional SQLite table stores templates without rewriting history.

## Live COSMIC checklist

1. In **Templates**, create a title, collection and multiline body, e.g. `Hello {{name}}, server {{server}} will be available on {{date}}. Thanks {{name}}.` Save, close and reopen: the template must persist.
2. Click its title. Each field appears once. Fill values and inspect the preview; missing fields disable copying. Date is manual. Copy and paste into an editor: repeated name occurrences must match.
3. Templates always **copy only**, even when ordinary history clips use direct paste. No paste shortcut is sent for a template. Values are not saved into the template and closing the view clears them from that view's memory. Copied results may enter normal clipboard history.
4. Edit, go back and explicitly discard. Stored text must remain unchanged. Use **Create template** from a text clip's preview and verify its original contents remain unchanged.
5. Search title/body without accents; filter by collection. Rename and delete a collection: templates remain, first renamed and then unfiled. Clear history and apply retention: templates survive.
6. Export to a new JSON file, then import it. Nothing changes before confirmation. Test skip, keep both with new IDs, and replace existing IDs. Identical titles with different IDs are not conflicts.
7. Import malformed JSON, an unknown version or invalid template: no partial templates or collections may remain. Exporting to an existing file must fail without changing it.
8. Check compact/expanded popup, full window, both themes and your usual display scaling. Try Tab, Shift+Tab, Enter on buttons, Ctrl+F in the list and Escape. History copy shortcuts must not copy clips while editing templates. File choosers use the system portal; cancelling must re-enable transfer buttons.

## Limits

500 templates; 100-character titles; 64 KiB bodies; 32 unique fields; 40-character field names using Unicode letters/digits, `_`, `-`; 256 KiB expanded output; 16 MiB exportable library/import limit. Double braces are reserved for fields and invalid syntax is rejected. Single braces remain literal text. Exports contain templates and their collection names only, without history or OCR index. Files and database are local and unencrypted.

## Automated checks

```bash
cargo fmt --check
cargo test --locked
cargo build --locked
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
nebula-paste --render-preview /tmp/templates.png templates dark
nebula-paste --render-preview /tmp/templates-popup.png templates-popup light
nebula-paste --render-preview /tmp/templates-edit.png templates-edit-popup dark
```

Renders are isolated demos: no user history access or actual clipboard copy/import/export. They do not validate portals or the compositor.
