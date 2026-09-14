# Tester Nebula Paste 0.8 bêta

[English](TESTING-0.8.en.md)

## Installation et données

Retirer l’applet du panneau, installer cette version, puis la rajouter. Vérifier `nebula-paste --version` : `0.8.0-beta.1`. L’historique 0.7, les favoris, les collections et l’index OCR doivent rester présents. Une table SQLite supplémentaire stocke les modèles ; l’historique n’est pas réécrit.

## Parcours à essayer dans COSMIC

1. Dans **Modèles**, créer une réponse avec un titre, une collection et plusieurs lignes. Exemple : `Bonjour {{nom}}, le serveur {{serveur}} sera disponible le {{date}}. Merci {{nom}}.` Enregistrer, fermer et rouvrir : le modèle doit être conservé.
2. Cliquer sur son titre. Chaque champ n’apparaît qu’une fois. Remplir les valeurs et vérifier l’aperçu ; tant qu’un champ manque, la copie reste indisponible. La date se remplit manuellement. Copier, puis coller dans un éditeur : les deux occurrences du nom doivent être identiques.
3. Les modèles utilisent **la copie seule**, même si le collage direct est activé pour les copies ordinaires. Aucun raccourci de collage n’est envoyé pour un modèle. Les valeurs saisies ne sont pas ajoutées au modèle ; fermer la vue les efface de la mémoire de cette vue. Le résultat copié peut être capturé normalement dans l’historique.
4. Modifier le texte, revenir en arrière puis abandonner explicitement. Vérifier que le modèle enregistré n’a pas changé. Depuis l’aperçu d’une copie texte, choisir **Créer un modèle** : le contenu original doit rester inchangé.
5. Rechercher un titre ou un mot sans accent (par exemple `reponse` pour `réponse`). Filtrer par collection. Renommer puis supprimer une collection : ses modèles doivent rester présents, déplacés vers le nouveau nom puis sans collection. Vider l’historique et appliquer la rétention : aucun modèle ne doit disparaître.
6. Exporter vers un nouveau fichier `.json`, puis l’importer. Avant confirmation, rien ne change. Essayer chacune des politiques de conflit : ignorer, conserver les deux avec de nouveaux identifiants, remplacer les identifiants existants. Les titres identiques avec des identifiants différents ne sont pas des conflits.
7. Importer un fichier JSON mal formé, d’une version inconnue ou avec un modèle invalide : aucune entrée ni collection partielle ne doit rester. Essayer d’exporter vers un fichier existant : l’opération doit être refusée sans modifier le fichier.
8. Vérifier panneau compact, popup agrandi et fenêtre complète, thèmes clair et sombre, mise à l’échelle habituelle. Tester Tab, Maj+Tab, Entrée sur un bouton, Ctrl+F dans la liste, et Échap. Les raccourcis de copie de l’historique ne doivent pas déclencher une copie pendant l’édition d’un modèle. Les boîtes de choix de fichiers doivent s’ouvrir via le portail système ; annuler doit rendre les boutons disponibles.

## Limites

500 modèles ; titre de 100 caractères maximum ; corps de 64 Kio ; 32 champs distincts, noms de 40 caractères maximum (lettres/chiffres Unicode, `_`, `-`) ; résultat de 256 Kio maximum ; bibliothèque exportable/import de 16 Mio maximum. Les doubles accolades sont réservées aux champs ; une syntaxe incorrecte est refusée. Une accolade simple reste du texte. L’export contient uniquement les modèles et leurs collections, sans copies ni index OCR. Les fichiers et la base sont locaux, non chiffrés.

## Vérification automatisée

```bash
cargo fmt --check
cargo test --locked
cargo build --locked
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
nebula-paste --render-preview /tmp/templates.png templates dark
nebula-paste --render-preview /tmp/templates-popup.png templates-popup light
nebula-paste --render-preview /tmp/templates-edit.png templates-edit-popup dark
```

Les rendus sont des démonstrations isolées : ni lecture de l’historique utilisateur, ni copie, import ou export réel. Ils ne prouvent pas le fonctionnement des portails ou du compositeur.
