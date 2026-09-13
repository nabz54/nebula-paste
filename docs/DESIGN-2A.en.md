# Nebula Paste — Dust Capture (2A)

[Français](DESIGN-2A.md) · [SVG proof sheet](identity-2a.svg) · [Roadmap](ROADMAP.en.md)

Selected direction: an asymmetric nebula, a stellar opening and two detached capture corners. The cloud expresses Nebula; the corners refer to capturing clipboard content. They do not introduce a screenshot feature.

## Integrated artwork

- `resources/io.github.nebulapaste.NebulaPaste.svg`: launcher and header; flat deep violet, lavender and turquoise, without glow.
- `resources/io.github.nebulapaste.NebulaPaste-symbolic.svg`: single-color panel mark, recolored by the theme. The central opening is transparent, never painted white.
- `docs/identity-2a.svg`: actual SVGs at 16, 24 and 32 px on light and dark backgrounds. View at 100% for native sizes.

Both icons share the same geometry on a 24-unit grid with 2-unit margins. Recognition does not depend on colored details. The panel button retains libcosmic sizing and interactions; pausing retains its existing indicator.

Source installation and RPM packaging already use these filenames. Rebuilding and installing updates the launcher, the embedded header icon and the panel. No additional runtime asset or download is required.

## Theme and validation

Application surfaces, selections and text keep the COSMIC system theme. Data and preferences are unchanged.

Both SVGs were rendered and inspected at 16, 24 and 32 px against light and dark backgrounds. The 16 px rendering has less detail; silhouette and corners take priority. Live COSMIC validation is still required: panel sizes, scaling, both themes and pause state. Older screenshots do not depict this identity.
