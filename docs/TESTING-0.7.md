# Tester la 0.7 sur Fedora COSMIC

[English](TESTING-0.7.en.md)

Cette bêta ajoute des tables SQLite ; elle ne réécrit pas le contenu des copies de la 0.6. Pour tester une mise à jour, ferme l’applet et sauvegarde ton dossier de données `nebula-paste` avant installation. Les tests automatisés ne remplacent pas une session COSMIC réelle.

1. **Migration :** démarre avec un historique 0.6 comprenant favoris et catégories. Vérifie le contenu, les dates et les favoris. Les catégories apparaissent comme collections.
2. **Collections :** crée « Travail » et une collection vide ; redémarre et vérifie leur présence. Renomme « Travail ». Essaye un nom déjà utilisé : l’opération doit échouer sans modifier les copies. Supprime une collection après confirmation : ses copies et favoris restent dans l’historique.
3. **Classement :** ouvre l’historique complet et glisse la poignée d’une carte vers une collection de la barre latérale. Vérifie le filtre et recopie l’élément. Texte, image ou URI doivent rester identiques. Vérifie aussi le classement depuis l’aperçu sur une petite fenêtre.
4. **Recherche OCR :** copie une image avec du texte lisible. Active la recherche d’images dans les préférences et attends la fin de l’indicateur. Recherche un mot reconnu, puis combine avec le filtre Images et une collection. Aucune nouvelle copie de texte ne doit apparaître. Vérifie le texte dans l’aperçu.
5. **Vie de l’index :** redémarre ; les résultats doivent rester disponibles. Change de langue OCR et attends la nouvelle indexation. Désactive l’option : les mots présents seulement dans les images ne donnent plus de résultats. Réactive puis utilise Réindexer.
6. **Suppression pendant OCR :** lance une indexation et supprime l’image, ou désactive l’indexation. Un résultat tardif ne doit pas recréer la copie ni son index. Une pause empêche les nouveaux traitements automatiques ; le traitement déjà lancé peut finir.
7. **COSMIC :** teste panneau horizontal et vertical, thèmes clair/sombre, échelles 100/150/200 %, popup compact et fenêtre redimensionnée. Fermer l’historique doit préserver la capture ; le collage direct facultatif doit viser l’application choisie.
8. **RPM :** installe le paquet de ta version Fedora, ajoute l’applet, puis vérifie le lanceur Historique, l’icône et l’OCR sans paquet Tesseract. La désinstallation doit conserver les données. Le RPM fourni est un paquet amont de test.

Signale la version de Fedora/COSMIC, l’échelle, les étapes et le résultat attendu/observé. Utilise des copies de test sans informations personnelles dans les captures partagées.
