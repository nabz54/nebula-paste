# Installation et mise à jour

[English](INSTALL.en.md)

Cible du paquet : **Fedora 44 x86_64 avec COSMIC**. Paquet amont non signé ; aucune autre distribution n'est actuellement annoncée comme testée. La 1.0.0-rc.1 est une candidate, pas encore une version stable. Les sources sont publiques et leur clonage ne demande aucune connexion GitHub.

## Télécharger

Quand la candidate est publiée, son RPM et `SHA256SUMS` sont joints à [la release](https://github.com/nabz54/nebula-paste/releases/tag/v1.0.0-rc.1). Avant publication, utiliser l'artefact `nebula-paste-fedora44-rpm` d'une compilation réussie de la branche `release/1.0-rc1` dans GitHub Actions ; sa récupération peut demander une connexion GitHub.

Depuis une release publiée, ces commandes fonctionnent sans compte GitHub :

```bash
mkdir -p ~/Téléchargements
repertoire=$(mktemp -d "$HOME/Téléchargements/nebula-1.0-XXXXXX")
cd "$repertoire"
base='https://github.com/nabz54/nebula-paste/releases/download/v1.0.0-rc.1'
curl --fail --location --output nebula-paste-1.0.0~rc.1-1.fc44.x86_64.rpm "$base/nebula-paste-1.0.0~rc.1-1.fc44.x86_64.rpm" &&
curl --fail --location --output SHA256SUMS "$base/SHA256SUMS"
```

Le manifeste contient également le RPM source, qui n'est pas nécessaire à l'installation. Vérifier précisément le binaire téléchargé :

```bash
grep -F '  nebula-paste-1.0.0~rc.1-1.fc44.x86_64.rpm' SHA256SUMS | sha256sum -c -
```

Continuer uniquement si la vérification indique OK.

## Éviter l'ancienne installation utilisateur

Retirer l'applet du panneau et fermer l'historique avant la mise à jour. Si le programme a été installé avec `scripts/install.sh`, exécuter `bash scripts/uninstall.sh` depuis ce projet. L'historique et les préférences sont conservés. Sans les sources, déplacer les seuls fichiers de lancement dans un dossier de sauvegarde :

```bash
pkill -x nebula-paste || true
sauvegarde=$(mktemp -d "$HOME/nebula-ancienne-installation-XXXXXX")
applications="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
for fichier in \
  "$HOME/.local/bin/nebula-paste" \
  "$applications/io.github.nebulapaste.NebulaPaste.desktop" \
  "$applications/io.github.nebulapaste.NebulaPaste.History.desktop"
do
  if [ -f "$fichier" ]; then mv -- "$fichier" "$sauvegarde/"; fi
done
hash -r
```

Ne pas supprimer les dossiers de données/configuration. Les lanceurs utilisateur masquent ceux du RPM même si `/usr/bin/nebula-paste --version` affiche la nouvelle version. Si un raccourci pointe vers `~/.local/bin/nebula-paste`, le remplacer par `/usr/bin/nebula-paste --toggle`.

## Installer ou mettre à jour

Dans le dossier contenant le RPM vérifié :

```bash
sudo dnf install './nebula-paste-1.0.0~rc.1-1.fc44.x86_64.rpm'
/usr/bin/nebula-paste --version
```

Le résultat attendu est `Nebula Paste 1.0.0-rc.1`. DNF met à niveau le paquet 0.9 existant. Ajouter Nebula Paste dans les réglages des applets du panneau COSMIC. Si le lanceur n'est pas actualisé, fermer puis rouvrir la session. Vérifier le processus actif :

```bash
for pid in $(pgrep -x nebula-paste); do readlink "/proc/$pid/exe"; done
```

Attendu : `/usr/bin/nebula-paste`, sans `(deleted)`. Le moteur OCR français/anglais est embarqué. Le collage direct est facultatif et nécessite `wtype` ainsi que le support du protocole clavier virtuel ; la copie suivie de Ctrl+V reste disponible.

## Données, sauvegarde et désinstallation

Historique : `${XDG_DATA_HOME:-~/.local/share}/nebula-paste/history.sqlite3`. Préférences : `${XDG_CONFIG_HOME:-~/.config}/nebula-paste/settings.conf`. Aucune nouvelle migration de schéma n'est introduite par cette candidate.

Avant une mise à niveau, exporter l'historique dans Préférences → Sauvegarde et diagnostic et exporter les modèles séparément. Les références de fichiers ne sauvegardent pas les fichiers eux-mêmes ; les exports et la base ne sont pas chiffrés.

Pour désinstaller : retirer l'applet, fermer sa fenêtre puis `sudo dnf remove nebula-paste`. Les données utilisateur restent conservées. La réinstallation d'une ancienne version n'est pas une garantie de compatibilité future : conserver les exports.

## Dépannage

- Flou : comparer le popup au calendrier sur le même fond, puis agrandir/réduire. Les paramètres COSMIC des fenêtres et des applets sont distincts. Le correctif porte sur les popups ; le rendu de la fenêtre séparée reste à valider.
- Mauvaise version : vérifier le chemin du processus ci-dessus et `rpm -q nebula-paste` ; un lanceur local ou une instance ancienne peut subsister.
- Historique absent : utiliser le même compte et les mêmes dossiers XDG ; ne pas lancer l'applet avec sudo.
- Rapport : joindre version, version Fedora/COSMIC et étapes de reproduction. Le diagnostic intégré exclut le contenu des copies ; relire les captures avant de les partager.
