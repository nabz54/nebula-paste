# Testing Nebula Paste 0.9 beta

[Français](TESTING-0.9.md)

## Backup and restoration

Preferences → **Backup and diagnostics**. Includes clip bytes, timestamps, favorites, collections (even empty) and existing OCR indexes. Excludes templates, settings and files referenced by URIs (only the URI is retained). Templates keep their own export. Archives use private 0600 permissions but are not encrypted. Existing destinations cannot be overwritten, including after the file chooser returns.

1. Create text clips, an indexed image, a favorite and an empty collection. Export, then change a clip’s metadata.
2. Import: the preview shows additions, conflicts, favorites, collections and size. Nothing changes until an explicit policy is chosen and restoration confirmed.
3. Conflicts mean identical content (SHA-256 identifier). **Keep local metadata** preserves dates, collections, favorite flags and OCR. **Use backup metadata** applies these fields from the archive. Unrelated clips stay untouched; new clips are added under either policy.
4. Confirmation explicitly disables the age limit and persists that before the transaction, preventing immediate deletion after restore or restart. Reapply your preferred retention using the affected-count preview. Favorites and templates remain exempt from age-based deletion.
5. With image search disabled, imported OCR is omitted. Re-enabling search reconstructs it locally.
6. Try truncated JSON, unknown versions, invalid hashes and capacity overflow: restoration must be all-or-nothing. Limits: 500 clips, 128 MiB of content, 128 collections and a 256 MiB JSON archive. Restore never evicts other clips.
7. Escape cancels the preview. History copy shortcuts must not act while working in preferences, templates or collections.

Diagnostics include only version, generic OS, architecture, counts and OCR/retention settings. No content, titles, collection names, paths, clip identifiers or environment variables.

## Verification and benchmarks

```bash
cargo fmt --check
cargo test --locked
cargo build --locked
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
bash scripts/benchmark.sh
```

Synthetic histories only: 1/100/500 clips, 200 accent-insensitive queries. OCR uses repository fixtures, three fresh processes per language, engine startup included. Development profile; results depend on CPU and load and do not measure compositor latency.

| Live scenario | Fedora 44 + COSMIC |
|---|---|
| Installation, upgrade and restart | Pending |
| Compact/expanded popup, resized separate history | Pending |
| Panel edges, multiple monitors, 100/150/200% scaling | Pending |
| Light/dark, transparency enabled/disabled | Pending |
| File portal, cancellation, keyboard and focus | Pending |
| Full-history restoration followed by restart | Pending |

Automated tests cannot prove behavior in a real COSMIC session. This remains a beta.
