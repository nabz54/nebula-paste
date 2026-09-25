# Paquet Fedora / Fedora package

## Français

Paquet amont de test, non signé et non publié dans les dépôts officiels Fedora. Le workflow **Fedora RPM** construit sur Fedora 44 et fournit un RPM binaire, un RPM source et `SHA256SUMS` dans son artefact. Une connexion GitHub peut être nécessaire pour télécharger un artefact Actions ; le clonage des sources publiques reste libre.

Pour construire sur Fedora avec Rust/Cargo 1.93 ou plus récent :

```bash
sudo dnf install git rust cargo gcc-c++ cmake make rpm-build pkgconf-pkg-config libxkbcommon-devel wayland-devel fontconfig-devel freetype-devel desktop-file-utils python3 tar gzip findutils diffutils
bash scripts/build-rpm.sh
```

Le script exige des modifications suivies déjà commitées. Il archive uniquement `HEAD`, récupère les dépendances Cargo verrouillées, prépare les licences, puis lance `rpmbuild` hors ligne. Les RPM se trouvent dans `dist/rpm/`. Construis sur ta version de Fedora pour éviter les différences de bibliothèques système.

Après téléchargement et extraction de l’artefact, vérifie les empreintes puis installe le RPM binaire (pas le `.src.rpm`) :

```bash
sha256sum -c SHA256SUMS
sudo dnf install ./nebula-paste-1.0.0~rc.1-1.fc44.x86_64.rpm
```

Retire l’applet du panneau avant la mise à jour et ajoute-la après. Si tu utilisais `scripts/install.sh`, désinstalle d’abord cette installation avec `bash scripts/uninstall.sh` pour éviter que le binaire utilisateur masque `/usr/bin/nebula-paste`. L’historique est conservé par la désinstallation normale ; ne supprime pas le dossier de données. Le RPM ne modifie pas automatiquement ton panneau. `wtype` est une recommandation pour le collage direct.

## English

This is an unsigned upstream test package, not an official Fedora repository package. **Fedora RPM** builds on Fedora 44 and uploads binary/source RPMs plus `SHA256SUMS`. Downloading Actions artifacts may require a GitHub login; cloning public sources does not.

Use the build commands above on Fedora with Rust/Cargo 1.93 or newer. The script packages committed `HEAD` only, vendors locked Cargo dependencies and licenses, then builds offline inside rpmbuild. Results are in `dist/rpm/`. Build on your Fedora release to match its system libraries.

After extracting the artifact, verify checksums and install the binary RPM using the commands above, not the `.src.rpm`. Remove the applet from the panel before updating and add it back afterwards. If previously installed with `scripts/install.sh`, run `bash scripts/uninstall.sh` first to avoid a user binary shadowing `/usr/bin/nebula-paste`. Normal uninstall preserves history; do not remove the data directory. RPM installation does not edit your panel. `wtype` is recommended for optional direct paste.
