# Validation 1.1 — développement

[English](TESTING-1.1.en.md)

Ne pas confondre cette branche avec une release publiée. Le flou, le focus, le défilement physique et les raccourcis doivent être essayés dans une session COSMIC.

- Bandeau avec 0, 1, 100 et 500 copies : défiler au début/milieu/fin ; aucune carte vide permanente ; cliquer copie bien la carte visée.
- Alt+flèches traverse l’ancienne limite de page et garde la sélection visible. Espace ouvre l’aperçu ; Échap le ferme sans perdre la sélection. Dans la recherche, Espace insère un espace ; dans les éditeurs, il ne déclenche pas d’aperçu. Entrée conserve son action de copie configurée.
- Après défilement, rechercher, filtrer, changer de tri et vider une recherche : sélection et position cohérentes, pas d’index hors bornes.
- Trois tailles : tester à 600/940 unités et avec mise à l’échelle 100/150/200 %. Sous 600, repli compact.
- Masquer/afficher filtres et collections depuis les préférences ; redémarrer et vérifier la persistance. Masquer les filtres réinitialise le filtre de type.
- Choisir une collection par défaut puis ouvrir l’applet ; supprimer cette collection et vérifier le retour à l’historique. Vérifier récent/ancien et compact/élargi après relance.
- Vérifier clair/sombre, thème changé à chaud, retour d’aperçu et passage compact/élargi.
- Noter matériel, versions, temps d’ouverture/recherche/défilement et mémoire à 100/500 copies. Les miniatures restent en cache ; ne pas annoncer une limite mémoire non mesurée.

Automatisation : formatage, tests des préférences anciennes/invalides et persistantes, navigation sur historique long, sélection/aperçu/recherche et isolation des éditeurs ; compilation et rendus natifs via CI. Résultats réels à renseigner avant stabilisation.
