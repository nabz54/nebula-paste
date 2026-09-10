# Nebula Paste 0.6 — essais Fedora COSMIC

[English](TESTING-0.6.en.md)

Version : **0.6.0-beta.1**. Les tests automatisés ne valident pas le focus et les interactions avec le compositeur. Ne pas utiliser les résultats de la 0.5 comme preuve pour cette version.

## Installer la branche

Depuis un clone du projet sans modifications locales :

```bash
git fetch origin
git switch v0.6-cosmic
bash scripts/install.sh
```

Retirer puis ajouter l’applet au panneau pour lancer le nouvel exécutable. Si COSMIC conserve l’ancien processus, fermer puis rouvrir la session. L’historique et les préférences existants sont conservés.

## Contrôles automatisés

```bash
cargo test --locked
cargo build --locked
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
```

Captures sans session graphique (données de démonstration uniquement) :

```bash
cargo run --locked -- --render-preview /tmp/nebula-popup-light.png popup light
cargo run --locked -- --render-preview /tmp/nebula-popup-dark.png popup dark
cargo run --locked -- --render-preview /tmp/nebula-history-light.png full light
```

## Essais manuels

| Essai | Résultat attendu |
|---|---|
| Panneau en haut, bas, gauche, droite | Icône lisible ; popup ancré à l’icône, contenu accessible |
| Thème clair, sombre, accent personnalisé ; changement pendant l’ouverture | Fonds, textes, boutons, icônes et champs suivent le thème |
| Échelle 100 %, 150 %, 200 % | Textes lisibles, pas de contrôles coupés ; vérifier aussi le petit écran |
| Ouvrir l’applet | Liste de cinq copies par page, popup de 360 unités logiques |
| Ouvrir l’historique complet | Le popup ferme ; une fenêtre distincte affiche les mêmes données |
| Fermer l’historique avec sa croix, Échap ou Alt+F4 | L’applet reste active ; une nouvelle copie est capturée |
| `nebula-paste --history` deux fois | Une seule fenêtre d’historique |
| `nebula-paste --toggle` depuis l’historique | Retour au popup sans seconde fenêtre |
| Redimensionner l’historique | Grille adaptée ; liste compacte sélectionnable dans Préférences |
| Ctrl+F, Alt+flèches, Ctrl+1…5 dans le popup, Entrée | Recherche et sélection cohérentes |
| Copier puis Ctrl+V dans un éditeur | Texte exact ; fenêtre fermée sauf préférence contraire |
| Collage direct / terminal, avec wtype | Fermeture avant Ctrl+V / Ctrl+Maj+V ; aucun collage dans Nebula |
| Favoris, collection, suppression puis annulation | Métadonnées conservées dans les deux vues |
| Pause, OCR FR/EN, rétention avec Appliquer | Fonctions de la 0.5 toujours disponibles |
| Langue FR/EN | Nouvelles commandes et infobulles traduites |

Pour un problème, indiquer la version de Fedora/COSMIC, l’échelle, la position du panneau, le scénario et le résultat observé. Utiliser du texte de test sans données personnelles pour les captures.

## Périmètre

Cette version conserve le stockage, les préférences et le mécanisme FR/EN existants. Migration Fluent/cosmic-config, snippets et recherche dans l’OCR des images restent des évolutions futures.
