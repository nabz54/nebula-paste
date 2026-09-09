#!/usr/bin/env bash
set -euo pipefail
data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"
rm -f -- "$HOME/.local/bin/nebula-paste" "$data_dir/applications/io.github.nebulapaste.NebulaPaste.desktop" "$data_dir/icons/hicolor/scalable/apps/io.github.nebulapaste.NebulaPaste.svg"
for license in Tesseract-Apache-2.0.txt Leptonica-BSD.txt tessdata_fast-Apache-2.0.txt Nebula-Paste-MPL-2.0.txt THIRD-PARTY.md; do
    rm -f -- "$data_dir/licenses/nebula-paste/$license"
done
rmdir -- "$data_dir/licenses/nebula-paste" 2>/dev/null || true
echo 'Fichiers de l’applet retirés. Retire aussi Nebula Paste du panneau COSMIC.'
echo 'Ton historique est conservé dans le dossier de données nebula-paste.'
