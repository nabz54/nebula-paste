# Testing 0.7 on Fedora COSMIC

[Français](TESTING-0.7.md)

This beta adds SQLite tables without rewriting 0.6 clip content. Close the applet and back up your `nebula-paste` data directory before testing an upgrade. Automated tests do not replace a live COSMIC session.

1. **Migration:** start with 0.6 history containing favorites and categories. Check content, dates and favorites. Categories should appear as collections.
2. **Collections:** create “Work” and an empty collection; restart and check persistence. Rename Work. Try an existing name: failure must leave clips unchanged. Confirm collection deletion: clips and favorites remain in history.
3. **Filing:** in full history, drag a card handle onto a sidebar collection. Check the filter and copy the item again. Text, image or URI bytes should remain unchanged. Also test collection assignment in details on a narrow window.
4. **OCR search:** copy a readable text image. Enable image search in Preferences and wait for indexing. Search a recognized word, then combine with Images and collection filters. No extra text clip should appear. Review recognized text in image details.
5. **Index lifecycle:** restart and check search persistence. Change OCR language and wait for reindexing. Disable the option: words found only inside images should stop matching. Enable again and use Reindex.
6. **Deletion during OCR:** start indexing then delete the source or disable indexing. Late results must not recreate a clip or index. Capture pause prevents new automatic OCR work; existing work may finish.
7. **COSMIC:** test horizontal/vertical panels, light/dark themes, 100/150/200% scaling, compact popup and resized history. Closing history must preserve capture. Optional direct paste should reach your selected application.
8. **RPM:** install a package built for your Fedora release, add the applet, and check the history launcher, icon and OCR without system Tesseract. Uninstall must preserve data. The provided RPM is an upstream test package.

Report Fedora/COSMIC versions, display scale, steps and expected/observed results. Use non-sensitive sample clips in shared screenshots.
