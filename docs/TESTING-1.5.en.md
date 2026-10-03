# Nebula Paste 1.5 — boards and automation

Beta in preparation, bundled with colored filters in PR #14.

## Collection boards

Open **Library**, choose a collection and select **View: board**. List/cards remain available; the choice persists per collection. Clips, notes and templates share the board without duplication. Search and scope filters still apply.

Create up to 12 columns, rename them and reorder them using the header arrows. Move cards with their ← / → buttons (cross-column drag-and-drop is not part of this first version). **Unassigned** holds items without a column. Confirmed column deletion preserves every item. Boards scroll horizontally and share the library's 40-item pagination; column counts include all matching items, not just the current page.

## Local rules

Open **Library → Rules and cleanup…**. Choose a type, optional matching text and a destination collection. Text is required when no type is selected. Matching is literal and case/accent insensitive; no regex, network or assumed source-application detection.

Saved rules apply to subsequent captured **unfiled, unpinned clips**. Already filed clips are not moved automatically. Several rules pointing to one destination are compatible; different destinations are conflicts and leave the clip unchanged. Rules can be edited, disabled or removed; removal never moves existing clips.

For existing history, preview matches and conflicts before applying unambiguous matches. Stale previews are rejected. Undo restores the previous collections until another application or app exit, refusing to overwrite subsequently changed filing. Automatic filing of new clips can be corrected manually; batch undo only covers the previewed batch.

## Expiration by use

Disabled by default. Choose 1–3650 inactive days and excluded collections, preview affected clips, then explicitly confirm deletion and activation. Deletion is permanent; back up first if needed.

The timer uses the latest capture/recapture or successful Nebula copy. Previewing is not usage. History copies, favorite shortcuts and plain-text copying count even while clipboard capture is paused. Transformations producing different content do not arbitrarily refresh their sources.

Favorites, notes and templates are protected, as are excluded collections. Cleanup runs on refresh and about once a minute while the applet is running. Existing age-based retention and capacity limits remain independent; these exclusions do not disable them.

## Migration and backup

The additive SQLite migration creates a private, consistent `*.pre-1.5.sqlite3` backup beside an existing legacy database before changes. Later opens do not overwrite it. Column/rule/preferences persist in the database; collection rename/deletion updates their references.

History restoration disables expiration by use before refreshing. Existing history/note/template JSON exports retain their original scope and do not back up board/rule metadata. For a full organization backup, quit the applet and back up its data directory. The pre-migration file contains the state before 1.5, not later edits.

## Fedora COSMIC checks

1. Create a collection with a clip, note and template; add/rename/reorder columns and move each item type. Reopen and restart: view and positions persist.
2. Delete a column/collection: preserve items. Rename a collection: board/rules/exclusions follow it.
3. Verify colored filters, selected state, keyboard focus, light/dark themes and colors disabled.
4. Test text/type rules, case/accents, unfiled/filed/pinned clips and conflicting destinations.
5. Preview/apply/undo a batch; change a rule before applying or move an item before undoing: stale/conflicting operations must be rejected.
6. Use a backed-up test database for expiration: old/recently copied/pinned clips, notes, templates and excluded collections. No deletion before confirmation; disable and restart.
7. Undo bulk deletion of cards assigned to columns. Restore history: expiration by use becomes disabled.

Automated tests and headless native previews do not replace real COSMIC session testing.
