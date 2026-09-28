# Nebula Paste 1.3 — actions and shortcuts

Prepared version: **1.3.0-beta.1**, branch `feature/1.3-actions`, PR #12. Not released yet.

## Usage

- Open **Text actions…** from a text clip preview, or select text clips/notes in the library and choose **Actions…**. Images and file clips cannot be assembled.
- Choose Original, uppercase, lowercase, trim whitespace at each line boundary or remove blank lines. Source items are preserved; output is plain text.
- Reorder with arrows, remove parts, choose a separator and inspect the preview. Copy the output or open a new note before saving it.
- **Prepare queue** freezes the current parts and transformation, without assembly separators. **Copy next** copies one item; press Ctrl+V in the target application. Progress advances only on success. **Previous** moves the cursor back so the previous item can be copied again. No keystroke is injected into the target application.
- Assign five fixed favorite slots. **Change** cycles through pinned clips; identifiers persist across restarts. Deleted or unpinned favorites become unavailable without silently substituting another clip.
- Choose click behavior separately for compact, shelf and window surfaces. In the library this applies to clicking a clip title; explicit Copy buttons keep copying. Selection alone never copies.
- Edit internal shortcuts in **Actions and keyboard**, then Apply per action. Empty disables a binding. Internal duplicates and reserved editing shortcuts are rejected; changes persist.

## Global COSMIC shortcuts

Assign these commands manually to custom desktop shortcuts. Nebula does not alter COSMIC configuration or detect desktop-wide shortcut conflicts.

```sh
/usr/bin/nebula-paste --toggle
/usr/bin/nebula-paste --history
/usr/bin/nebula-paste --copy-favorite 1
/usr/bin/nebula-paste --queue-next
/usr/bin/nebula-paste --queue-back
```

Favorite slots range from 1 to 5. The applet must be running; the queue must have been prepared in the current session. Commands request copying, never automatic pasting. Actual shortcut delivery still needs testing on Fedora COSMIC. Copy errors appear in the Actions screen.

## On-device validation

1. Transform/assemble accented text, braces, shell commands and whitespace. Check output and untouched originals.
2. Check separators, ordering, note creation and preservation of an existing unsaved note draft.
3. Queue A/B/C; copy A then B, go back and copy B again. C must remain next. Editing the assembly must not change a prepared queue.
4. Assign a favorite, restart and use its binding. Delete it: no other clip may replace it.
5. Reject duplicate shortcuts; verify custom bindings persist. Typing in note/template/preferences editors must not trigger copying.
6. Check all click behaviors and invoke global commands from another application.
7. Test dark/light themes, compact layout, focus and compositor blur in a real COSMIC session.

## Limits

Assembly: 100 parts and 256 KiB including transformations. Notes: 64 KiB. Queue and assembly are session-only. Whitespace trimming/blank-line removal normalize line endings; Original preserves literal text. Automatic sequential paste, screen capture and color picking are outside this version.
