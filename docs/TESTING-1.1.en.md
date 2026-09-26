# 1.1 validation — development

[Français](TESTING-1.1.md)

This branch is not a published release. Blur, focus, physical scrolling and shortcuts require a COSMIC session.

- Shelf with 0, 1, 100 and 500 clips: scroll to start/middle/end; no persistent blank cards; clicking copies the intended card.
- Alt+arrows cross the old page boundary and reveal selection. Space opens preview; Escape closes it without losing selection. Space types normally in search and must not open previews in editors. Enter retains the configured copy action.
- After scrolling, search, filter, change sorting and clear search: selection and offset remain consistent and in bounds.
- Three sizes at 600/940 logical units and 100/150/200% scaling. Below 600, compact fallback.
- Toggle filters/collections in preferences and restart to check persistence. Hiding filters clears the type filter.
- Select an opening collection; delete it and check fallback to history. Check newest/oldest and compact/expanded after restarting.
- Check light/dark, live theme changes, preview return and layout switching.
- Record hardware, versions, opening/search/scrolling latency and memory with 100/500 clips. Thumbnails remain cached; do not claim an unmeasured memory bound.

Automation: formatting, old/invalid/persistent settings, long-history navigation, selection/preview/search and editor isolation; build and native renders in CI. Record real-session results before stabilizing.
