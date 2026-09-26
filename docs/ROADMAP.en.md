# Nebula Paste roadmap

[Français](ROADMAP.md)

Goals agreed on September 26, 2026, inspired by Supaste workflows and adapted to Rust/COSMIC. Versions are targets without promised dates. 1.0.0-rc.1 is published; stable 1.0 requires the real-session checks in [TESTING-1.0](TESTING-1.0.en.md).

Three surfaces share data and actions: compact applet for quick access, expanded shelf for visual browsing, full window for organization. All retain the 2A identity and native COSMIC theme.

| Version | Goal | Scope |
|---|---|---|
| 1.0 stable | Reliable foundation | RC validation, blur, focus, multiple displays, RPM migration, backup and OCR. Baseline measurements. |
| 1.1 | Navigation and layout | Continuous shelf, card sizes, optional filters/collections, Space preview, keyboard navigation, default view/collection and sort order. |
| 1.2 | Collections and notes | List/card views, persistent notes separate from history, editing, collection ordering, multi-selection/move and unified search. |
| 1.3 | Actions and shortcuts | Internal/global/favorite shortcuts, conflict handling, text transformations, combining clips, sequential copy and click behavior. |
| 1.4 | Capture and images | Region capture + OCR, color picker, text extraction, image resizing/conversion. Prototype optional muted video playback in an open preview. |
| 1.5 | Boards and automation | Editable Kanban columns, local classification rules with preview/undo; unused-clip expiry with exclusions. |
| 1.6 | Distribution and consolidation | Update notification, update flow adapted to the Fedora distribution channel, performance, accessibility and complete workflow validation. |

## 1.1 delivery

First development batch: continuous shelf constructing widgets near the visible region; small/medium/large cards; optional filters and collections; Space preview only for events not consumed by an input; Alt+arrow navigation; persistent layout, opening collection and ordering preferences. Compact list and full window keep pagination.

Existing image thumbnails remain cached in memory: widget virtualization is not on-demand image decoding. Measure memory, opening, search and scrolling at 100 and 500 clips before deciding whether an additional bounded cache is needed. Ordering initially supports newest/oldest; manual ordering follows collection work.

See [TESTING-1.1](TESTING-1.1.en.md). Headless screenshots do not validate Wayland focus or compositor blur. Unit tests alone do not establish stability.

## Data principles

- Expiring history stays separate from retained notes/templates; retention excludes favorites.
- Collections contain items; Kanban columns add organization. Switching views never duplicates data.
- Local automatic rules explain classification, disclose conflicts and allow correction. Source-app rules require reliable source information.
- Migrations are additive or explicitly versioned, backed up and covered by restore checks.

## Prototype before committing

Direct/sequential paste, global shortcuts, screen capture and color picking require COSMIC/Wayland capability checks and fallbacks. Copy followed by Ctrl+V remains available. One-click installation requires a selected Fedora distribution channel and authentication mechanism. Floating shelf, inline text expansion and local background removal remain exploratory until feasibility, resource use and quality are validated.

Sync, remote services and AI are not required. Fixes, performance, accessibility and FR/EN documentation belong in every release.
