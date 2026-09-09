# Contribuer à Nebula Paste

Nebula Paste est un applet Rust pour COSMIC sur Linux. Consulte le README pour les dépendances de compilation de ta distribution. Le moteur OCR nécessite aussi CMake, Make et un compilateur C/C++ ; ses sources et modèles sont inclus dans `vendor/`.

## Développer

```bash
cargo build --locked
cargo run --locked -- --preview
```

Le mode aperçu utilise des contenus de démonstration en mémoire. Pour tester l’intégration au panneau et les échanges Wayland, utilise une session COSMIC et suis la procédure d’installation du README.

## Vérifier une modification

```bash
cargo fmt --check
cargo test --locked
bash -n scripts/install.sh scripts/uninstall.sh scripts/check-embedded-ocr.sh
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
```

Le dernier contrôle utilise le binaire produit par `cargo build`. Pour une modification de l’interface, vérifie aussi l’aperçu et le comportement réel dans COSMIC. Indique dans ta contribution les contrôles réalisés et ceux qui restent à faire. Le fichier `VALIDATION.md` décrit les résultats de la version distribuée ; il ne garantit pas les modifications ultérieures.

## Proposer une contribution

Crée une branche dédiée et explique le problème, le changement apporté et les vérifications effectuées dans ta pull request. Pour un bug, précise la distribution, la version de COSMIC et les étapes de reproduction. Utilise des exemples synthétiques dans les captures et les tests : ne joins pas ton historique de presse-papiers.

Conserve `Cargo.lock` pour rendre les dépendances reproductibles. Toute mise à jour des sources ou modèles OCR doit aussi mettre à jour `vendor/SHA256SUMS`, la documentation et les licences concernées.

Le projet est sous MPL-2.0, voir `LICENSE`. Les composants embarqués conservent leurs licences dans `vendor/licenses/`.
