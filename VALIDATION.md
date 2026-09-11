# Validation courante — 0.5.0-dev.1, 10 septembre 2026

## 0.6.0-beta.1

- `cargo check --locked --all-targets` réussi avec Rust 1.93.0 sous Ubuntu 24.04 : compilation du moteur OCR embarqué et vérification des types des tests de régression incluses.
- `cargo fmt --check`, `git diff --check`, syntaxe shell, lecture TOML et SVG : réussis.
- Ces contrôles ne prouvent ni l’exécution des tests, ni l’édition de liens release, ni le fonctionnement dans Fedora COSMIC. La CI est configurée pour exécuter les tests et vérifier l’OCR embarqué.
- [Procédure de test Fedora](docs/TESTING-0.6.md).

Les résultats ci-dessous concernent les versions précédentes.


- Compilation de développement Linux réussie avec Rust 1.98.1 et dépendances verrouillées.
- **33 tests réussis** : 4 bibliothèque/préférences/langues, 17 application/OCR/transferts, 12 historique/intégrité/rétention/restauration.
- `cargo fmt --check` et syntaxe des scripts Bash validés.
- OCR réel anglais, français et combiné validé avec PATH vide, modèles système ignorés et sans liaison dynamique à Tesseract/Leptonica.
- Captures natives produites et inspectées en français/anglais, grille 940 px, liste 470 px, préférences 470 px et grille étroite 360 px. En-tête et pagination corrigés après inspection.
- Patch utilisateur intégré avec corrections : restauration transactionnelle, texte/code inchangé en copie brute, pause monotone, rétention appliquée explicitement, fichier temporaire de préférences unique.

**Limites :** build de développement, pas de build release ; aucun essai dans une session Fedora COSMIC réelle. Placement du popup, redimensionnement par le compositeur, interaction Wayland, retour de focus, icône symbolique selon le thème et glisser-déposer restent à contrôler sur ce bureau. Les captures hors écran ne valident pas ces échanges.

Les blocs suivants conservent l’historique des validations antérieures et sont remplacés, pour l’état courant, par les résultats ci-dessus.

---

# Validation 0.4.0 — 9 septembre 2026

Environnement : Linux x86_64, Ubuntu 24.04, Rust 1.98.1. Compilation native depuis les sources embarquées, dépendances Rust verrouillées dans Cargo.lock.

| Vérification exécutée | Résultat |
| --- | --- |
| cargo build --locked | Exécutable compilé et lié en profil dev |
| cargo test --locked | 18 tests réussis, aucun échec ni test ignoré |
| cargo fmt --check | Réussi |
| bash -n sur les trois scripts | Réussi |
| --version | Nebula Paste 0.4.0 |
| OCR anglais embarqué | « NEBULA PASTE 12345 » reconnu |
| OCR français embarqué | « Été à Nancy : déjà prêt, élève, café. » reconnu |
| OCR français + anglais | Même phrase et accents reconnus |
| Transparence | Texte noir sur fond transparent reconnu après composition sur fond blanc |
| Image vide et langue invalide | Erreurs explicites vérifiées |
| Autonomie OCR | Réussite avec PATH vide et TESSDATA_PREFIX pointant vers un dossier vide |
| Liaison native | Aucune entrée DT_NEEDED pour libtesseract ou Leptonica |
| Capture de l’aperçu | Vrai rendu logiciel, mention « OCR intégré · hors ligne », 940 × 661 pixels |

Les 18 tests comprennent les 8 tests existants de l’historique et 10 tests sur les actions, le glisser-déposer, l’intégration à l’interface et l’OCR. Les tests OCR utilisent réellement les modèles embarqués, y compris le modèle français.

Le contrôle reproductible scripts/check-embedded-ocr.sh lance la commande --ocr avec un PATH vide et un dossier de modèles système vide. Il contrôle aussi les dépendances ELF directes. Les trois langues passent sur l’exécutable final, sans diagnostics parasites de codecs ou polices de débogage.

Tesseract 5.5.1 et Leptonica 1.85.0 sont liés statiquement. Les modèles eng/fra de tessdata_fast 4.1.0 sont incorporés avec include_bytes!. Les archives originales, les licences et les sommes SHA-256 sont fournies dans vendor/. Le programme n’appelle pas la commande tesseract et ne télécharge aucun modèle.

## Limites

Le profil release et l’installation dans une session Fedora COSMIC réelle n’ont pas été exécutés ici. L’installateur compile en release sur la machine cible. L’archive contient les sources complètes et les modèles, sans binaire précompilé.

Le moteur demande coopérativement l’arrêt après 30 secondes de reconnaissance ; il n’est plus isolé dans un processus OCR qu’on pourrait tuer. Les limites de taille/dimensions, le travail en arrière-plan et l’invalidation des résultats dont l’image a été supprimée sont conservés.

Les bibliothèques de base Linux, la bibliothèque standard C++ et les dépendances graphiques COSMIC restent des dépendances système. Seule la chaîne OCR est embarquée. wtype reste facultatif pour le collage direct.

Le placement du popup, le focus du collage direct et les échanges Wayland avec d’autres applications restent à tester dans le bureau cible. Le schéma SQLite, le chemin de l’historique et l’identifiant de l’applet ne changent pas.

La capture de l’aperçu a été régénérée pour 0.4. Les captures de la grille à quatre et huit éléments proviennent de 0.3, dont le design est conservé.

## Conditionnement GitHub

L’archive Leptonica est livrée en deux parties pour respecter la limite de transfert de la connexion GitHub. Les empreintes des fichiers livrés ont été vérifiées ; leur concaténation est identique octet pour octet à l’archive originale et son extraction avec `tar -xzf` réussit. `build.rs` effectue cette concaténation avant extraction. La compilation Rust et les 18 tests précédents n’ont pas été relancés après cette adaptation du conditionnement.

## Proposition visuelle 0.5.0-dev.1

SVG parsés et planche PNG rendue puis inspectée. Contrastes des textes principal et secondaire sur les quatre surfaces de la planche vérifiés : minimum 6.49:1. Versions Cargo.toml/Cargo.lock concordantes ; syntaxe des scripts install/uninstall vérifiée avec bash -n. Rust/Cargo indisponibles dans cet environnement au moment de cette modification : compilation, tests et affichage réel COSMIC non exécutés pour cette proposition. Les résultats 0.4 ci-dessus ne constituent pas une validation de la 0.5.
