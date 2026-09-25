# Journal des modifications

[English](CHANGELOG.md)

## 1.0.0-rc.1 — Candidate à la version stable

- Vue élargie en bandeau : une rangée d’aperçus, recherche intégrée, compteurs de filtres, pagination et actions au clic droit ou via le bouton d’aperçu. Vue compacte conservée.
- Flou natif COSMIC pour les popups compacts et élargis, suivant les réglages du thème (correctif de la PR #8).
- Numéro distinct de la bêta 0.9 : mise à niveau RPM et diagnostic de version sans ambiguïté.
- Publication préparée pour joindre le RPM Fedora 44 et ses empreintes à la release, après réussite des tests et de la construction.
- Installation, migration depuis `~/.local/bin`, dépannage et critères de validation documentés en français et anglais.
- Aucun changement du format des données dans cette candidate. La validation des parcours et de la fenêtre séparée reste à terminer avant 1.0 stable.

## 0.9.0-beta.1 — Fiabilité et maîtrise des données

Sauvegarde/restauration de l’historique avec conflits explicites et transaction ; diagnostic sans contenu privé ; aperçu de rétention ; transparence native COSMIC, fenêtre d’historique redimensionnable et isolation des raccourcis. Mesures recherche/OCR reproductibles. Les essais COSMIC réels restent à faire. [Guide de test](docs/TESTING-0.9.md).

## 0.8.0-beta.1 — Modèles réutilisables

- Nouvelle section **Modèles** dans le popup et la fenêtre complète : créer, modifier, rechercher, classer et supprimer avec confirmation.
- Création depuis une copie texte ; modèles conservés séparément de l’historique, hors rétention et vidage.
- Champs `{{nom}}`, `{{date}}`, `{{serveur}}`, `{{ip}}` ou noms personnalisés : remplir une fois, vérifier l’aperçu, puis copier le résultat. La date est saisie manuellement.
- Import/export JSON avec sélecteur de fichiers système, aperçu et choix explicite en cas de conflit. Export vers un **nouveau fichier** ; aucun écrasement d’export existant.
- Identité 2A et interface française/anglaise suivant le thème COSMIC.

Bêta : les tests automatisés et rendus ne remplacent pas les essais dans ta session Fedora COSMIC. [Guide de test 0.8](docs/TESTING-0.8.md) · [Roadmap](docs/ROADMAP.md).

## 0.7.0-beta.1 — Recherche et collections

- Popup raccourci : cinq lignes de 72 unités, miniatures réduites et actions latérales ; fond opaque et filtres sans défilement horizontal.
- Recherche tolérante aux accents latins, aux ligatures françaises et aux accents décomposés, également dans le texte OCR ; contenu original préservé.
- Recherche OCR locale dans les images, désactivée par défaut ; index persistant limité à 16 384 caractères par image, sans copies supplémentaires.
- Traitement séquentiel en arrière-plan, sélection de langue, progression, texte reconnu dans l’aperçu et réindexation manuelle.
- Effacement de l’index à la désactivation et avec les copies supprimées ; rejet des résultats périmés après suppression ou changement de réglage.
- Collections persistantes, y compris vides : création, renommage atomique, suppression avec confirmation conservant les copies et favoris.
- Migration automatique des catégories 0.6, barre latérale adaptative et classement par glisser-déposer interne.
- Paquet RPM amont Fedora, dépendances Rust vendues, licences incluses et workflow de construction.
- Documentation et guide de test français/anglais ; tests de migration, recherche, conservation des données et cycle OCR.
- Bêta : validation interactive Fedora COSMIC encore nécessaire. Les extraits réutilisables sont reportés à une version ultérieure.

## 0.6.0-beta.1 — Intégration COSMIC

- Surfaces, contrôles, textes et icônes symboliques suivent le thème système.
- Popup compact de 360 unités logiques, cinq copies par page.
- Historique partagé dans une fenêtre séparée et redimensionnable, sans second moniteur ni second processus d’écriture.
- Commande `--history` et lanceur nécessitant l’applet active dans le panneau.
- La fermeture de l’historique conserve la capture ; le collage direct ferme la vue active.
- Infobulles bilingues, rendus clair/sombre, tests de cycle des fenêtres et contrôles CI.
- Historique, préférences et OCR embarqué 0.5 conservés, sans migration de données.
- Essais Fedora COSMIC à effectuer : [guide](docs/TESTING-0.6.md).

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
