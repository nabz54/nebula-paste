# Nebula Paste — applet pour COSMIC

**1.0.0-rc.1 — candidate à la version stable.** Flou natif des applets corrigé ; validation finale en cours. [Installation et mise à jour](docs/INSTALL.md) · [Validation 1.0](docs/TESTING-1.0.md).


[English](README.en.md) · [Changelog](CHANGELOG.fr.md) · [Contribuer](CONTRIBUTING.md)

Un gestionnaire de presse-papiers local en **Rust/libcosmic** pour Fedora COSMIC : textes, images, liens, couleurs, code et références de fichiers. Projet indépendant, sans affiliation à Supaste ou System76.

## Modèles réutilisables

- Nouvelle section **Modèles** dans le popup et la fenêtre complète : créer, modifier, rechercher, classer et supprimer avec confirmation.
- Création depuis une copie texte ; modèles conservés séparément de l’historique, hors rétention et vidage.
- Champs `{{nom}}`, `{{date}}`, `{{serveur}}`, `{{ip}}` ou noms personnalisés : remplir une fois, vérifier l’aperçu, puis copier le résultat. La date est saisie manuellement.
- Import/export JSON avec sélecteur de fichiers système, aperçu et choix explicite en cas de conflit. Export vers un **nouveau fichier** ; aucun écrasement d’export existant.
- Identité 2A et interface française/anglaise suivant le thème COSMIC.

Bêta : les tests automatisés et rendus ne remplacent pas les essais dans ta session Fedora COSMIC. [Guide de test 0.8](docs/TESTING-0.8.md) · [Roadmap](docs/ROADMAP.md).

## Installer depuis les sources

```bash
sudo dnf install git rust cargo gcc gcc-c++ cmake make pkgconf-pkg-config libxkbcommon-devel wayland-devel fontconfig-devel freetype-devel
git clone --branch main https://github.com/nabz54/nebula-paste.git
cd nebula-paste
bash scripts/install.sh
```

Rust 1.93 ou plus récent est requis. Le dépôt public se clone sans connexion GitHub. Retire l’ancienne applet du panneau avant de mettre à jour, puis ajoute **Nebula Paste** dans les réglages du panneau COSMIC. L’historique et les préférences existants sont conservés ; les catégories de la 0.6 deviennent des collections au premier démarrage.

Le moteur Tesseract, Leptonica et les modèles français/anglais sont incorporés dans le binaire. Aucun paquet Tesseract ni téléchargement de modèle à l’utilisation. La première compilation peut être longue.

## Utiliser

Dans le popup, **Agrandir / Réduire** bascule entre la liste compacte et la grille élargie, sans effacer la recherche ni les filtres. Le choix est enregistré. Les collections restent accessibles dans les onglets ; **Fenêtre séparée** ouvre l’historique indépendant. La surface du panneau conserve uniquement son icône. La largeur du popup peut être réduite par COSMIC si l’écran manque de place.

Clique sur l’icône du panneau, recherche ou filtre les copies, puis clique sur une carte pour recopier son contenu original. Ouvre **Historique** pour la fenêtre complète. Le lanceur « Nebula Paste — Historique » et `nebula-paste --history` nécessitent l’applet déjà active.

Dans **Préférences**, active la recherche dans les images. L’indexation traite une image à la fois en arrière-plan avec la langue OCR choisie. Les résultats rejoignent la recherche existante sans ajouter de copies de texte. L’aperçu de l’image affiche le texte reconnu pour vérification. Désactiver cette option efface l’index ; **Réindexer** permet de reprendre après une erreur. Une pause de capture suspend le démarrage de nouveaux traitements OCR automatiques, sans interrompre le traitement déjà lancé.

Crée une collection avec **Gérer les collections**, puis glisse une copie par sa poignée sur la collection dans la barre latérale. L’aperçu permet également de saisir une collection. Chaque copie appartient à une collection au maximum ; les favoris restent indépendants. Le dépôt depuis une application externe n’est pas pris en charge.

**Raccourcis :** Ctrl+F recherche, Alt+flèches sélection, Entrée copie, Ctrl+1…9 copie une entrée de la page, Ctrl+Maj+C copie le texte brut, Échap retour/fermeture. En mode copie seule, colle ensuite avec Ctrl+V. Le collage direct facultatif nécessite `sudo dnf install wtype` et le protocole clavier virtuel autorisé par COSMIC ; il vise l’application ayant le focus après fermeture.

## Données locales et limites

SQLite stocke l’historique et l’index OCR dans le dossier de données local de Nebula Paste, avec des permissions privées. Les données ne sont pas chiffrées et ne sont jamais envoyées à un service OCR. L’index est limité à 16 384 caractères par image et disparaît avec la copie source. Les favoris sont protégés de la rétention et du vidage de l’historique. Limites : 500 copies, 128 Mio de contenu, 16 Mio par copie, 128 nouvelles collections maximum (les catégories héritées restent conservées).

La recherche ignore la casse et les accents latins (par exemple « ete » retrouve « été ») et combine les mots avec les filtres de type, favoris et collection ; ce n’est pas une recherche approximative. Une reconnaissance peut être inexacte. Les fichiers restent des références à leur emplacement d’origine.

## Développer et tester

```bash
cargo fmt --check
cargo test --locked
cargo build --locked
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
```

`nebula-paste --preview` utilise des données fictives et désactive la capture et la copie. L’OCR embarqué est en C/C++ ; l’application est en Rust. Sources, modèles, licences et empreintes sont dans `vendor/`. Voir [VALIDATION.md](VALIDATION.md) et le [guide de test](docs/TESTING-0.7.md) pour les limites de validation.
