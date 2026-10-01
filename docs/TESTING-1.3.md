# Nebula Paste 1.3 — actions et raccourcis

Version préparée : **1.3.0-beta.1**, branche `feature/1.3-actions`, PR #12. Pas encore publiée.

## Utilisation

- Depuis l’aperçu d’une copie textuelle : **Actions texte…**. Depuis la bibliothèque : sélectionner des copies textuelles et/ou notes, puis **Actions…**. Images et fichiers sont refusés dans un assemblage.
- Choisir Original, majuscules, minuscules, nettoyage des espaces en début/fin de chaque ligne ou suppression des lignes vides. Les originaux restent intacts ; le résultat est une copie textuelle.
- Réordonner les textes avec les flèches, retirer des éléments, choisir le séparateur et vérifier l’aperçu. Copier le résultat ou ouvrir une nouvelle note avant de l’enregistrer.
- **Préparer la file** fige les éléments et leur transformation courante, sans séparateur entre éléments. **Copier le suivant** copie un seul texte ; coller ensuite avec Ctrl+V dans l’application cible. La file avance seulement après réussite. **Reculer** replace le curseur sur l’élément précédent ; aucune frappe n’est injectée dans l’application cible.
- Dans **Actions**, affecter cinq emplacements de favoris. **Changer** parcourt les copies marquées comme favorites ; l’affectation est enregistrée par identifiant. Un favori supprimé ou retiré des favoris devient indisponible, sans substitution automatique.
- Choisir le comportement au clic par surface : compacte, bandeau et fenêtre. Dans la bibliothèque, cliquer le titre d’une copie applique ce comportement ; le bouton Copier reste explicite. Sélectionner ne copie pas.
- Personnaliser les touches dans **Actions et clavier**, puis Appliquer pour chaque action. Une valeur vide désactive l’action. Doublons internes et raccourcis d’édition réservés sont refusés. Les changements sont persistants.

## Raccourcis globaux dans COSMIC

La 1.3 expose des commandes destinées aux raccourcis personnalisés du bureau. Associer manuellement les touches souhaitées dans COSMIC ; l’application ne modifie pas la configuration du bureau et ne détecte pas ses conflits globaux.

```sh
/usr/bin/nebula-paste --toggle
/usr/bin/nebula-paste --history
/usr/bin/nebula-paste --copy-favorite 1
/usr/bin/nebula-paste --queue-next
/usr/bin/nebula-paste --queue-back
```

Les favoris acceptent les emplacements 1 à 5. L’applet doit fonctionner dans le panneau ; la file doit avoir été préparée pendant cette session. Ces commandes demandent une copie, jamais un collage automatique. Tester leur déclenchement réel sur Fedora COSMIC reste nécessaire. Les erreurs de copie sont visibles dans l’écran Actions.

## Validation sur le PC

1. Copier du texte avec accents, accolades, commandes et espaces ; transformer et assembler, puis comparer le résultat et les originaux.
2. Vérifier les séparateurs, les flèches, la création de note et la protection d’un brouillon de note existant.
3. Préparer une file A/B/C, copier A puis B, reculer et recopier B. Vérifier que C reste le suivant. Modifier ensuite l’assemblage : la file déjà préparée reste inchangée.
4. Affecter un favori à un emplacement, relancer l’applet puis utiliser sa touche. Supprimer ce favori : aucune autre copie ne doit le remplacer.
5. Affecter deux actions à la même touche : refus explicite. Modifier une touche, fermer/rouvrir et vérifier la persistance. Vérifier qu’une saisie dans les notes, modèles et préférences ne déclenche aucune copie.
6. Tester les trois comportements au clic dans chaque surface et les commandes globales depuis une autre application.
7. Vérifier les thèmes clair/sombre, l’affichage compact, le focus et le flou dans une session COSMIC réelle.

## Limites

Assemblage : 100 éléments et 256 Kio maximum, transformations comprises. Note : 64 Kio maximum. File et assemblage restent uniquement en mémoire jusqu’à l’arrêt. Les transformations normalisent les retours de ligne uniquement lorsqu’elles nettoient les espaces ou retirent les lignes vides. Original conserve le texte tel quel. Pas de collage séquentiel automatique, de capture de zone ni de pipette dans cette version.
