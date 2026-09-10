# Journal des modifications

[English](CHANGELOG.md)

## 0.5.0 — Non publiée en version stable

### Ajouts
- Identité originale : icônes couleur et symbolique, palette ardoise/cyan et planche visuelle.
- Grille adaptative et liste compacte, navigation clavier commune et poignées de glisser-déposer.
- Interface français/anglais et préférences persistantes ; langue OCR indépendante.
- Rétention configurable avec bouton Appliquer et protection des favoris.
- Copie en texte brut et Ctrl+Maj+C, sans modifier le texte ou le code.
- Pauses temporaires ou sans limite, compte à rebours et reprise automatique.
- Annulation de la dernière suppression pendant douze secondes, confirmation de copie et option pour rester ouvert.
- Documentation anglaise et changelogs bilingues.

### Corrections
- Restauration transactionnelle : métadonnées préservées, aucune recapture écrasée ni autre entrée évincée.
- Annulation expirée ou effacement global : aucune restauration possible.
- Pause basée sur une horloge monotone ; invalidation des transferts antérieurs.
- Le texte brut ne retire plus les balises d’un code source.
- Préférences écrites dans un fichier temporaire privé unique, puis remplacées atomiquement.

### État
Version de développement sur `v0.5-design`. Les contrôles sont détaillés dans VALIDATION.md ; la validation réelle sous Fedora COSMIC reste nécessaire avant une version stable.

## 0.4.0 — 2026-09-09
- Tesseract/Leptonica et modèles français/anglais embarqués ; OCR hors ligne sans exécutable Tesseract système.
- Diagnostic OCR en ligne de commande et script de vérification du moteur embarqué.
- Sources natives, empreintes et licences tierces incluses.

## 0.3.0
- Glisser-déposer sortant natif, collage direct facultatif avec wtype et OCR des images.
- Aperçus enrichis et navigation améliorée.

## 0.2.0
- Surfaces opaques, cartes rectangulaires et meilleure lisibilité.

## 0.1.0
- Premier applet Rust/COSMIC avec historique local, filtres et favoris.

Les anciennes entrées résument les étapes de développement et ne signifient pas qu’une release ou une étiquette GitHub existe pour chacune.
