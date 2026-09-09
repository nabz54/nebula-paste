#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
command -v cargo >/dev/null || { echo 'Installe Rust avec rustup : https://rustup.rs'; exit 1; }
if ! command -v pkg-config >/dev/null || ! pkg-config --exists xkbcommon; then
    echo 'Dépendance manquante : fichiers de développement de xkbcommon.'
    echo 'Fedora : sudo dnf install libxkbcommon-devel pkgconf-pkg-config'
    echo 'Debian/Ubuntu : sudo apt install libxkbcommon-dev pkg-config'
    exit 1
fi
for tool in cmake make gcc g++ tar sha256sum; do
    command -v "$tool" >/dev/null || { echo "Outil de compilation manquant : $tool"; echo 'Fedora : sudo dnf install cmake make gcc gcc-c++'; exit 1; }
done
cargo build --release --locked
install -Dm755 target/release/nebula-paste "$HOME/.local/bin/nebula-paste"
data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"
for license in vendor/licenses/*.txt; do
    install -Dm644 "$license" "$data_dir/licenses/nebula-paste/$(basename -- "$license")"
done
install -Dm644 LICENSE "$data_dir/licenses/nebula-paste/Nebula-Paste-MPL-2.0.txt"
install -Dm644 vendor/README.md "$data_dir/licenses/nebula-paste/THIRD-PARTY.md"
install -Dm644 resources/io.github.nebulapaste.NebulaPaste.svg "$data_dir/icons/hicolor/scalable/apps/io.github.nebulapaste.NebulaPaste.svg"
install -Dm644 resources/io.github.nebulapaste.NebulaPaste.desktop "$data_dir/applications/io.github.nebulapaste.NebulaPaste.desktop"
# A desktop entry uses its own escaping, not shell quoting.
desktop_exec="$HOME/.local/bin/nebula-paste"
desktop_exec="${desktop_exec//\\/\\\\}"
desktop_exec="${desktop_exec//\"/\\\"}"
desktop_exec="${desktop_exec//\$/\\\$}"
desktop_exec="${desktop_exec//\`/\\\`}"
desktop_exec="${desktop_exec//%/%%}"
{ while IFS= read -r line; do
    if [[ "$line" == Exec=* ]]; then printf 'Exec="%s"\n' "$desktop_exec"; else printf '%s\n' "$line"; fi
done < resources/io.github.nebulapaste.NebulaPaste.desktop; } > "$data_dir/applications/io.github.nebulapaste.NebulaPaste.desktop"
if command -v update-desktop-database >/dev/null; then update-desktop-database "$data_dir/applications"; fi
echo 'Installé. COSMIC : Paramètres → Bureau → Panneau → Applets → Ajouter → Nebula Paste.'
echo 'Si absent de la liste, déconnecte puis reconnecte ta session.'
printf 'Raccourci personnalisé conseillé : Super+V → %s --toggle\n' "$HOME/.local/bin/nebula-paste"

if ! command -v wtype >/dev/null; then
    echo 'Collage direct facultatif (Fedora) : sudo dnf install wtype'
fi
echo 'OCR français/anglais inclus dans l’exécutable ; aucune installation Tesseract nécessaire.'
