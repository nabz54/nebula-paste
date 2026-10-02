# Nebula Paste 1.4 — capture, images and type colors

Version under test: **1.4.0-beta.1**. These checks complement automated tests before daily use.

## Workflow

Open **Capture** in the applet or **Capture…** in the shelf. **Image workbench…** in an image clip’s preview opens a working copy of that original.

- **Capture…** opens the desktop’s native screenshot dialog. Choose its area, window or screen selection. Nebula closes its windows first and returns to the workbench afterwards.
- **Screen text…** captures in the same way, then runs the embedded offline OCR. Review and edit its result before copying or creating a note. Select French, English or both in preferences. If background OCR indexing is busy, retry **Extract text** when it finishes.
- **Pick color…** asks the desktop for a pixel and shows its swatch, HEX and RGB. Copy either format, or save a HEX color clip to history.
- **Import image…** accepts local PNG, JPEG and WebP files. It also provides a fallback when the screenshot portal is unavailable; the picker has no simulated fallback.

The desktop portal owns selection and permissions. Nebula requests an interactive capture; it does not force area selection instead of the desktop dialog. Portal-created screenshot files remain managed by the portal; Nebula never deletes arbitrary returned paths.

## Image workbench

Crop uses **X, Y, width and height in original-image pixels**. Changing crop width/height resets output dimensions to the crop rectangle. Then resize with a locked or free aspect ratio. Each preview is calculated from the original, never cumulatively from the previous preview.

Choose PNG, JPEG or WebP and **Build preview**. Changing settings disables copying/export until a new preview is built. **Reset** restores the original. PNG and lossless WebP preserve alpha. JPEG flattens transparency onto white, with 70/85/95 quality choices.

- **Copy image** sets the clipboard; paste with Ctrl+V. Normal active clipboard monitoring may then add it to history.
- **Add to history** explicitly saves the result without changing the clipboard.
- **Export…** writes a new file. Its extension must match the selected format; existing files are never overwritten.
- **Extract text** processes the currently displayed image. Clear existing OCR text first before re-running it, protecting manual corrections. Later image changes do not automatically refresh that text.

Originals stay intact. Closing the panel retains the workbench for this session; quitting the applet discards unsaved work. **Clear workbench…** asks for confirmation before another capture/image can replace it. An existing note draft is protected when creating an OCR note.

## COSMIC shortcuts

Manually assign these commands in desktop keyboard settings:

```sh
/usr/bin/nebula-paste --capture
/usr/bin/nebula-paste --capture-text
/usr/bin/nebula-paste --pick-color
```

The panel applet must be running. An occupied workbench is shown with a request to clear it rather than silently replaced. Paused clipboard monitoring does not prevent these explicit actions.

## Type colors

**Preferences → Colors by type**: Subtle (default), Vivid, Off. Lavender text, blue links, pink images, mint code, amber files, yellow notes; color clips retain their own sample. Type names remain visible. Desktop theme, compositor blur and native selection outlines retain their existing behavior.

## Fedora COSMIC checks

1. Capture an area, window and screen. Check the applet returns and is absent from the capture; test multiple screens/scales.
2. Cancel all three native dialogs: no history insertion or stuck busy state. With an unavailable portal, check the visible error and local-image import fallback.
3. OCR French/English, edit, copy and create a note. Confirm an existing note draft is preserved.
4. Crop/resize a known image with locked/free aspect ratio. Invalid or empty crops must fail without changing the original.
5. Export and reopen PNG/JPEG/WebP to inspect dimensions and alpha. Existing names and wrong extensions must fail without overwriting.
6. Pick a known color, compare HEX/RGB, save and find its color clip.
7. Test global commands, cancellation recovery, color modes and compact/expanded views in light/dark themes.

Limits: 16 MiB compressed input/output; output at most 8192 pixels per side and 16 megapixels; edited OCR text 256 KiB, notes 64 KiB. Native dialogs time out after three minutes; close the old dialog before retrying. Video recording/playback and background removal are not included. CI screenshots do not validate a real Wayland session.
