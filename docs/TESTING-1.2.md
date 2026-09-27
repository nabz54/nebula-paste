# Valider la 1.2 sur Fedora COSMIC

Cette branche prépare **1.2.0-beta.1**. La PR #11 n’est pas une release et ne remplace pas automatiquement la version installée.

## Parcours à tester

1. Ouvrir **Bibliothèque**, puis **Nouvelle note**. Saisir un titre, du texte avec `{{accolades}}`, une commande et des accents. Enregistrer, rouvrir, copier : le texte doit rester littéral.
2. Modifier une note sans enregistrer. Retour doit demander de confirmer l’abandon. Changer d’onglet puis revenir doit garder le brouillon ; ouvrir une autre note doit refuser de remplacer ce brouillon.
3. Créer une note depuis une copie textuelle. Vérifier que la copie originale existe encore. Vider l’historique hors favoris : notes et modèles doivent rester présents.
4. Créer deux collections, les réordonner avec les flèches et choisir une vue liste pour l’une, cartes pour l’autre. Fermer et relancer : ordre et vues doivent être conservés. Renommer puis supprimer une collection : les notes doivent rester présentes, sans collection après suppression.
5. Chercher un mot présent dans une copie, une note et un modèle. Vérifier les trois types, les recherches sans accents et le filtrage par collection.
6. Sélectionner des copies et notes, les déplacer ensemble, puis annuler. Supprimer après confirmation, puis annuler : contenu et favoris doivent revenir. Les modèles ne sont jamais sélectionnés par l’action « toutes les copies et notes ».
7. Exporter les notes, puis importer le fichier avec chaque politique de conflit : ignorer, conserver les deux, remplacer. Annuler un import doit laisser la base intacte.
8. Vérifier la bibliothèque et l’éditeur en thème clair/sombre, dans une fenêtre étroite et dans l’applet. Tester le focus clavier et le retour à la capture sans perte de brouillon.

## Limites explicites

- Historique, notes et modèles ont chacun leur export. L’export historique ne contient pas les notes.
- Les exports ne contiennent pas l’ordre ni le mode d’affichage des collections.
- Une seule opération groupée est annulable, en mémoire jusqu’à la prochaine opération groupée ou l’arrêt. Si un élément a été recréé ou reclassé entre-temps, l’annulation peut être refusée pour éviter d’écraser son nouvel état.
- Les notes sont éditées dans un écran dédié. Vue tableau/Kanban et synchronisation ne font pas partie de la 1.2.
- Les rendus automatiques utilisent les widgets natifs mais ne valident pas le flou ni le focus dans une session Wayland réelle.
