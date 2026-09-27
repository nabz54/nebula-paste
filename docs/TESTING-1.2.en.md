# Validate 1.2 on Fedora COSMIC

This branch prepares **1.2.0-beta.1**. PR #11 is not a release and does not automatically replace the installed version.

## Test scenarios

1. Open **Library → New note**. Save text containing `{{braces}}`, shell commands and accents. Reopen and copy it: the content must remain literal.
2. Edit without saving. Back must ask before discarding. Switching tabs and returning must preserve the draft; opening another note must refuse to replace it.
3. Create a note from a text clip. The original clip must remain. Clear unpinned history: notes and templates must survive.
4. Create two collections, reorder them with arrows and choose a different list/card view for each. Restart and verify persistence. Rename and delete a collection: notes survive and become unfiled after deletion.
5. Search a word appearing in a clip, note and template. Verify all three result types, accent-insensitive search and collection filtering.
6. Select clips and notes, move them together and undo. Confirm bulk deletion, then undo: content and favorite flags must return. Templates are excluded from bulk selection.
7. Export and import notes with each conflict policy: skip, keep both and replace. Canceling an import must leave stored data unchanged.
8. Check dark/light themes, a narrow window and the applet; verify keyboard focus and returning to capture without losing a draft.

## Explicit limits

- History, notes and templates each have separate exports. History export does not contain notes.
- Collection ordering and view preferences are not included in exports.
- Only the latest bulk operation can be undone, until another bulk operation or application exit. Undo may refuse to overwrite an item recreated or refiled in the meantime.
- Notes use a dedicated editing screen. Board/Kanban view and synchronization are outside 1.2.
- Automated renders use native widgets but do not validate blur or focus in a real Wayland session.
