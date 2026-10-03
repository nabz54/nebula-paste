# Nebula Paste 1.5 — tableaux et automatisation

Bêta en préparation, livraison groupée avec les filtres colorés (PR #14).

## Tableau d’une collection

Dans **Bibliothèque**, choisis une collection puis **Vue : tableau**. Les vues liste/cartes restent disponibles ; le choix est mémorisé par collection. Le tableau rassemble copies, notes et modèles sans les dupliquer. La recherche et les filtres restent actifs.

Ajoute jusqu’à 12 colonnes. Renomme-les, change leur ordre avec les flèches de leur en-tête et déplace une carte avec ses boutons ← / →. Cette première version utilise ces boutons, pas le glisser-déposer entre colonnes. La colonne **Sans colonne** recueille les éléments non affectés. La suppression d’une colonne demande confirmation et conserve tous ses éléments. Le tableau utilise un défilement horizontal et la pagination de la bibliothèque (40 éléments par page) ; les compteurs incluent tous les résultats, pas seulement la page visible.

## Règles locales

**Bibliothèque → Règles et nettoyage…**. Choisis un type, un texte recherché et une collection de destination. Le texte est facultatif si un type est choisi. Les correspondances sont littérales, sans distinction de casse ni d’accents ; pas de regex, de réseau ni d’identification supposée de l’application source.

Une règle enregistrée est active pour les prochaines copies capturées **sans collection et hors favoris**. Une copie déjà classée n’est pas déplacée automatiquement. Plusieurs règles vers une même destination sont compatibles ; plusieurs destinations différentes constituent un conflit et la copie reste en place. Active/désactive, modifie ou retire les règles selon tes besoins. Retirer une règle ne déplace aucune copie.

Pour l’historique existant, **Prévisualiser sur l’historique** montre les destinations et les conflits. **Appliquer les correspondances sans conflit** ne touche que les correspondances non ambiguës. Si les données/règles ont changé, il faut actualiser l’aperçu. **Annuler ce classement** restaure les collections précédentes jusqu’à une nouvelle application ou la fermeture. L’annulation refuse d’écraser un classement modifié depuis. Le classement automatique des nouvelles captures peut être corrigé manuellement ; son annulation n’est pas celle du lot prévisualisé.

## Nettoyage selon l’utilisation

Désactivé par défaut. Saisis 1 à 3650 jours d’inactivité et sélectionne les collections exclues. Prévisualise les copies concernées, puis confirme explicitement la suppression et l’activation. La suppression est définitive : fais une sauvegarde si nécessaire.

Le délai utilise le plus récent entre capture/recapture et copie réussie via Nebula. Ouvrir un aperçu ne compte pas comme utilisation. Les copies via l’historique, favoris rapides et texte brut sont suivies même si la capture est en pause. Les transformations créant un autre contenu ne rajeunissent pas arbitrairement leurs sources.

Favoris, notes et modèles sont toujours protégés par ce nettoyage. Les collections exclues le sont également. Le contrôle s’effectue lors d’une actualisation et environ chaque minute tant que l’applet tourne ; aucune suppression pendant son arrêt. La rétention par âge déjà présente et les limites de capacité restent indépendantes : les exclusions ici ne les désactivent pas.

## Migration et sauvegardes

La migration ajoute des tables SQLite. À la première ouverture d’une ancienne base, une copie cohérente `*.pre-1.5.sqlite3` est créée à côté avant migration, avec permissions privées. Elle n’est pas écrasée aux ouvertures suivantes. Les colonnes, règles et préférences sont conservées dans la base ; renommer/supprimer une collection met à jour leurs références.

Une restauration d’historique désactive l’expiration par utilisation avant actualisation. Les exports JSON d’historique/notes/modèles conservent leur périmètre existant : ils ne constituent pas une sauvegarde des colonnes/règles. Pour une sauvegarde complète de l’organisation, quitte l’applet et sauvegarde son dossier de données avant manipulation. Le fichier pré-migration conserve l’état antérieur à la 1.5, pas les modifications faites ensuite.

## Vérifications Fedora COSMIC

1. Créer une collection avec copie, note et modèle ; activer le tableau ; ajouter, renommer et réordonner les colonnes ; déplacer les trois types d’éléments. Fermer/revenir, puis relancer l’applet : positions et vue conservées.
2. Supprimer une colonne puis une collection : conserver les éléments ; vérifier les favoris. Renommer une collection : tableau/règles/exclusions suivent le nouveau nom.
3. Tester les filtres colorés en clair/sombre, sélection, clavier et option couleurs désactivées.
4. Créer une règle texte et une règle par type ; tester accents/casse, copie sans collection, copie déjà classée et favori. Ajouter une règle contradictoire : copie inchangée et conflit dans l’aperçu.
5. Prévisualiser/appliquer/annuler un lot. Changer les règles avant application et déplacer un élément après application : refus du résultat périmé ou de l’annulation conflictuelle.
6. Tester le nettoyage avec une base de test sauvegardée : éléments anciens, récemment copiés, favoris, notes, modèles, collection exclue. Vérifier l’absence de suppression avant confirmation, la désactivation et la persistance après relance.
7. Vérifier l’annulation de suppression groupée avec des cartes affectées à des colonnes. Restaurer un historique : nettoyage par utilisation désactivé.

Les tests automatiques et rendus natifs ne remplacent pas ces essais dans une session COSMIC réelle.
