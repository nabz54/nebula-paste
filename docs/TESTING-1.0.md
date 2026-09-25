# Validation 1.0

[English](TESTING-1.0.en.md)

La 1.0.0-rc.1 prépare la stable. Ne pas confondre un test automatisé réussi avec un essai réel sous COSMIC.

## État constaté

- Correctif du flou des popups de la PR #8 : compilation, tests et RPM réussis sur le commit `3aec578b0da85ebb221ff6fdad95fc18f01a20f1` ; retour utilisateur positif le 24 septembre 2026 après installation du paquet corrigé.
- Ce retour ne valide pas tous les écrans, thèmes, fenêtres séparées ou parcours de restauration.
- Tests de cette candidate : consulter les exécutions associées à son commit. Ne pas réutiliser les résultats de la PR #8 pour annoncer la candidate validée.

## Parcours avant stable

Noter la date, `rpm -q nebula-paste cosmic-comp cosmic-panel`, le type de session et les facteurs d'échelle. Aucun contenu personnel n'est nécessaire au rapport.

| Parcours | Critère | État |
|---|---|---|
| Installation neuve RC | Ajout au panneau, capture, relance de session | À tester |
| Mise à niveau 0.9 → RC | Historique, favoris, collections, modèles et préférences conservés | À tester |
| Migration depuis installation locale | Un seul processus `/usr/bin/nebula-paste`, lanceurs corrects | À tester avec RC |
| Popup compact/élargi | Flou conforme au calendrier, recherche/filtres conservés, réouverture | Correctif confirmé par utilisateur ; RC à vérifier |
| Fenêtre séparée | Dimensions, fermeture, thème et transparence | À tester |
| Clavier et copie | Recherche, navigation, copie texte/image, collage Ctrl+V et collage direct facultatif | À tester |
| Thème et écrans | Clair/sombre, flou activé/désactivé, panneaux et échelles utilisés | À tester |
| Sauvegarde/restauration | Export séparé historique/modèles, aperçu, conflits, redémarrage, annulation | À tester |
| OCR | Français/anglais, désactivation et réindexation | À tester |
| Usage prolongé | Plusieurs semaines sans perte de données, plantage ou blocage connu | À documenter |

Suivre les scénarios détaillés des guides [0.8](TESTING-0.8.md) et [0.9](TESTING-0.9.md). Utiliser des copies fictives pour les essais d'importation et de rétention.

## Automatisation

`cargo fmt --check`, `cargo test --locked`, `cargo build --locked`, validation des fichiers desktop et `scripts/check-embedded-ocr.sh`. Sur la branche release, le RPM Fedora 44 est construit après les tests ; sa construction vérifie également l'OCR embarqué. Sur main, la publication dépend des tests et du RPM et joint les paquets et empreintes au même tag.

## Décision stable

Clore les défauts bloquants, renseigner les essais ci-dessus et les limitations restantes, puis mettre ensemble les versions Cargo/RPM et les changelogs à 1.0.0. Aucun changement de version à stable automatique. Les paquets actuels restent non signés ; aucun support d'autres versions Fedora n'est promis.

## Bandeau élargi

Vérifier une rangée de grandes vignettes et la pagination avec plus de quatre copies. Tester recherche, filtres et compteurs, copie au clic, actions au clic droit ou via « Aperçu / actions », favoris, suppression et annulation. Vérifier la navigation clavier, le retour en vue compacte et le repli automatique sur petit écran. Contrôler les messages d’erreur, la pause, les thèmes clair/sombre et le flou natif dans une session COSMIC. Les captures automatiques ne valident pas le compositeur.
