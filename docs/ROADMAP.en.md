# Nebula Paste roadmap

[Français](ROADMAP.md)

These are targets, not shipped features or promised dates. Version 0.7 remains a beta pending Fedora COSMIC validation.

## 0.8 — Identity and reusable templates

**Implemented in 0.8.0-beta.1:** 2A identity, persistent templates, editing, literal fields, search and import/export with conflict handling. Live Fedora COSMIC validation remains required before a stable release.

**Delivered features:**

1. Template storage separate from clipboard history: stable ID, title, body, optional collection, creation/update timestamps. Additive migration; history retention and clearing must never delete templates.
2. Templates view in popup and full window: create, edit, search title/body, file, confirm deletion. Create from a text clip without changing the original.
3. Explicit `{{name}}`, `{{date}}`, `{{server}}`, `{{ip}}` fields: form before copying, repeated occurrences share one value, final preview, values not persisted by default. Date remains a user-filled field initially. No shell expansion or command execution.
4. Versioned JSON import/export of templates only: size/count limits, full validation before writes, conflict preview, no silent replacement. Atomic export and transactional import.
5. French/English labels, help, changelog and testing guide.

**Exit criteria:** migration and history preservation tests; persistent CRUD; repeated/missing/Unicode fields and malformed syntax; import/export round trips and failures without partial writes; accent-insensitive search; keyboard navigation and compact/expanded views on COSMIC. Check the identity at actual panel sizes and with both themes.

## 0.9 — Reliability and data control

**Implemented in the 0.9 beta branch:** backup/restore, diagnostics, retention impact, transparency, resizing, keyboard isolation and reproducible measurements. Live COSMIC acceptance remains pending.

**Proposed:** strengthen placement, resizing, keyboard focus and theme changes; measure search and OCR with a full history; history backup/restore with preview and conflict handling; clarify retention controls; diagnostic exports excluding clip and template contents.

**Exit criteria:** interrupted/invalid restore tests, favorites and collections preserved, reproducible measurements, documented Fedora/COSMIC matrix, no known blocker on main workflows.

## 1.0 — Stable release

**In preparation: 1.0.0-rc.1.** User-confirmed popup blur fix, distinct version, RPM publication after tests and bilingual migration guides. [Remaining validation](TESTING-1.0.en.md).

**Proposed:** reproducible RPM installation and updates, data-preserving upgrades, complete FR/EN documentation, troubleshooting and uninstall instructions, explicit supported versions based on testing.

**Exit criteria:** several weeks of real usage, no known blocking defects, fresh installation and upgrades checked, published limitations. Timing depends on validation.

## After 1.0

Possible synchronization and extensions require a separate decision. No remote service is required by this roadmap.
