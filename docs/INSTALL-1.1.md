# Installer Nebula Paste 1.1.0-beta.1

[English](INSTALL-1.1.en.md)

Bêta pour **Fedora 44 x86_64 / COSMIC**, paquet amont non signé. La 1.0.0-rc.1 reste disponible séparément. Exporter l’historique et les modèles avant les essais. Cette version ne modifie pas le schéma des données ; les nouvelles préférences sont additives.

Lorsque [la release](https://github.com/nabz54/nebula-paste/releases/tag/v1.1.0-beta.1) est publiée, télécharger et vérifier le binaire sans connexion GitHub :

```bash
(
set -euo pipefail
dossier=$(mktemp -d "$HOME/nebula-1.1-XXXXXX")
cd "$dossier"
base='https://github.com/nabz54/nebula-paste/releases/download/v1.1.0-beta.1'
paquet='nebula-paste-1.1.0.beta.1-1.fc44.x86_64.rpm'
curl -fL "$base/$paquet" -o "$paquet"
curl -fL "$base/SHA256SUMS" -o SHA256SUMS
grep -F "  $paquet" SHA256SUMS | sha256sum -c -
sudo dnf install "./$paquet"
)
```

Le nom du fichier utilise des points ; sa version interne RPM conserve `1.1.0~beta.1` pour trier correctement les préversions. Le manifeste contient aussi le RPM source, inutile pour installer l’applet.

Déconnecter puis reconnecter la session COSMIC afin de relancer l’applet. Vérifier :

```bash
/usr/bin/nebula-paste --version
for pid in $(pgrep -x nebula-paste); do readlink "/proc/$pid/exe"; done
```

Attendu : `Nebula Paste 1.1.0-beta.1` et `/usr/bin/nebula-paste`, sans `(deleted)`. Si une installation dans `~/.local/bin` masque le RPM, suivre [la migration](INSTALL.md#éviter-lancienne-installation-utilisateur).

Dans les préférences, choisir le bandeau élargi, les tailles de cartes, les filtres/collections affichés et la collection d’ouverture. Essayer [les parcours 1.1](TESTING-1.1.md). Le rendu et le focus doivent être confirmés dans une session réelle ; il s’agit d’une bêta.
