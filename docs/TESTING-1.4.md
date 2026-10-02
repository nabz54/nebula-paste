# Nebula Paste 1.4 — capture, images et couleurs

Version préparée : **1.4.0-beta.1**, PR #13. La fusion et la publication sont distinctes de cette préparation.

## Parcours

Ouvre **Capture** dans l’applet, ou **Capture…** dans le bandeau. Depuis l’aperçu d’une image de l’historique, **Retoucher / capturer…** ouvre la même interface avec une copie de l’original.

- **Capturer…** ouvre le dialogue natif du bureau. Choisis sa sélection de zone, de fenêtre ou d’écran. Nebula ferme ses fenêtres avant d’ouvrir le dialogue puis revient dans l’atelier.
- **Texte de l’écran…** capture de la même façon, puis lance l’OCR embarqué. Le texte apparaît dans un éditeur avant toute copie : corrige-le, copie-le ou crée une note. La langue se choisit dans les préférences (français, anglais ou les deux). Si l’indexeur OCR est déjà occupé, relance **Extraire le texte** après sa fin.
- **Pipette…** demande un pixel au bureau et affiche son échantillon, son HEX et son RGB. HEX/RGB copient le format choisi ; **Enregistrer la couleur** ajoute une copie HEX à l’historique.
- **Importer une image…** accepte les fichiers PNG, JPEG et WebP locaux. C’est aussi le repli si la capture native n’est pas disponible ; la pipette n’a pas de repli simulé.

Le portail COSMIC gère la sélection et les autorisations. Nebula demande une capture interactive, sans forcer une zone à la place du dialogue du bureau. Le fichier produit par le portail reste géré par ce dernier : Nebula ne supprime pas arbitrairement un chemin qu’il reçoit.

## Atelier d’images

Le recadrage utilise **X, Y, largeur et hauteur en pixels de l’original**. Changer la largeur/hauteur du recadrage remet la taille de sortie à celle du rectangle. Les dimensions finales peuvent ensuite être liées proportionnellement ou libres. Les transformations sont toujours recalculées depuis l’original, jamais cumulées sur l’aperçu précédent.

Choisis PNG, JPEG ou WebP, puis **Calculer l’aperçu**. Toute modification des réglages désactive copie/export jusqu’au nouveau calcul. **Réinitialiser** revient à l’image originale. PNG et WebP conservent la transparence ; WebP est sans perte. JPEG remplace la transparence par du blanc et propose les qualités 70, 85 et 95.

- **Copier l’image** prépare le presse-papiers ; colle ensuite avec Ctrl+V. Le suivi normal du presse-papiers peut alors l’ajouter à l’historique s’il est actif.
- **Ajouter à l’historique** enregistre explicitement le résultat, sans changer le presse-papiers.
- **Exporter…** écrit un nouveau fichier du format sélectionné. L’extension doit correspondre ; un fichier existant n’est jamais écrasé.
- **Extraire le texte** utilise l’image affichée. Le texte déjà présent doit être effacé avant de refaire une extraction, afin de protéger les corrections manuelles. Modifier ensuite l’image ne recalcule pas automatiquement ce texte.

Les originaux restent intacts. Fermer le panneau conserve l’atelier dans la session ; quitter l’applet abandonne ce travail non enregistré. **Effacer l’atelier…** demande confirmation avant de libérer la place pour une nouvelle capture/image. Un brouillon de note déjà ouvert reste protégé lorsqu’on demande une nouvelle note OCR.

## Raccourcis COSMIC

Associer manuellement ces commandes aux touches souhaitées dans les paramètres du bureau :

```sh
/usr/bin/nebula-paste --capture
/usr/bin/nebula-paste --capture-text
/usr/bin/nebula-paste --pick-color
```

L’applet doit être en cours d’exécution dans le panneau. Si un atelier contient déjà un résultat, la commande le montre et demande de le vider ; elle ne remplace pas le travail en cours. La pause de l’historique n’empêche pas ces actions explicites.

## Couleurs par type

**Préférences → Couleurs par type** : Discrètes (défaut), Accentuées, Désactivées. Texte lavande, liens bleus, images roses, code menthe, fichiers ambre, notes jaunes ; les copies de couleur utilisent leur propre échantillon. Les types restent nommés. Le thème général, le flou du compositeur et le contour de sélection COSMIC restent gérés comme auparavant.

## Vérifications sur Fedora COSMIC

1. Capturer une zone de texte, une fenêtre et un écran ; vérifier le retour à l’applet, son absence dans la capture et le fonctionnement sur deux écrans/échelles différentes.
2. Annuler les trois dialogues : aucun nouvel élément dans l’historique, aucun état occupé bloqué. Tester un portail absent : erreur visible et import local utilisable.
3. Extraire du français avec accents et de l’anglais, corriger le texte, copier et créer une note. Vérifier qu’une note en cours n’est pas remplacée.
4. Recadrer une image connue puis redimensionner avec et sans proportions. Tester un rectangle vide/hors image : refus, original inchangé.
5. Exporter PNG/JPEG/WebP, rouvrir chaque résultat et vérifier dimensions/transparence. Tenter un nom déjà utilisé et une extension incorrecte : refus sans écrasement.
6. Prélever une couleur connue, comparer HEX/RGB, enregistrer puis retrouver la copie dans le filtre Couleurs.
7. Tester les commandes globales, la reprise après annulation, les modes de couleurs et les vues compacte/élargie en clair/sombre.

Limites : import et sortie compressée 16 Mio ; sortie 8192 pixels par côté et 16 mégapixels ; texte OCR édité 256 Kio, notes 64 Kio. Le dialogue natif expire après trois minutes sans réponse ; fermer alors l’ancien dialogue avant de relancer. Pas d’enregistrement vidéo, de lecture vidéo ni de suppression de fond dans cette livraison. Les captures CI ne valident pas une session Wayland réelle.
