# Roadmap Nebula Paste

[English](ROADMAP.en.md)

Objectifs approuvés le 26 septembre 2026, inspirés des parcours Supaste et adaptés à Rust/COSMIC. Les numéros ci-dessous sont des objectifs sans date promise. La 1.0.0-rc.1 est publiée ; la 1.0 stable dépend des essais réels décrits dans [TESTING-1.0](TESTING-1.0.md).

Trois surfaces partagent données et actions : applet compacte pour l’accès rapide, bandeau élargi pour le parcours visuel, fenêtre complète pour l’organisation. L’identité 2A et le thème natif COSMIC restent communs.

| Version | Objectif | Périmètre |
|---|---|---|
| 1.0 stable | Socle fiable | Validation RC, flou, focus, multi-écran, migration RPM, sauvegarde et OCR. Mesures de référence. |
| 1.1 | Navigation et rendu | Défilement continu, cartes réglables, filtres/collections masquables, aperçu Espace, navigation clavier, vue/collection et tri par défaut. |
| 1.2 | Collections et notes | Vues liste/cartes, notes persistantes distinctes de l’historique, édition, collections réordonnables, sélection/déplacement groupés, recherche commune. |
| 1.3 | Actions et raccourcis | Raccourcis internes/globaux et favoris, conflits, transformations de texte, assemblage, copie séquentielle, comportement au clic. |
| 1.4 | Capture et images | Capture de zone + OCR, pipette, extraction de texte, redimensionnement/conversion. Prototype de lecture vidéo muette et désactivable dans l’aperçu ouvert. |
| 1.5 | Tableaux et automatisation | Kanban, colonnes éditables, règles locales de classement avec aperçu/annulation ; expiration selon dernière utilisation avec exclusions. |
| 1.6 | Distribution et consolidation | Notification de version, parcours de mise à jour adapté au canal Fedora, performances, accessibilité et validation complète. |

## Livraison 1.1

Premier lot en développement : bandeau défilant avec construction des widgets proches de la zone visible ; tailles petite/moyenne/grande ; filtres/collections masquables ; aperçu Espace lorsque l’événement n’est pas consommé par un champ ; navigation Alt+flèches ; réglages de vue, collection et ordre persistants. La liste compacte et la fenêtre complète conservent leur pagination.

Les miniatures existantes restent mises en cache en mémoire : la virtualisation des widgets ne constitue pas encore un décodage d’images à la demande. Mesurer mémoire, ouverture, recherche et défilement à 100 et 500 copies avant de décider d’un cache borné supplémentaire. L’ordre configurable commence par récent/ancien ; le réordonnancement manuel attend le travail sur les collections.

Validation : [TESTING-1.1](TESTING-1.1.md). Les captures hors session ne valident ni le focus Wayland ni le flou du compositeur. Pas de déclaration de stabilité sur la seule base des tests unitaires.

## Principes de données

- Historique périssable séparé des notes/modèles conservés ; les favoris ne disparaissent pas par rétention.
- Une collection contient des éléments ; ses colonnes Kanban sont une organisation supplémentaire. Changer de vue ne duplique pas les données.
- Les règles automatiques locales doivent expliquer le classement, annoncer les conflits et permettre une correction. L’application source ne sera proposée que si l’information est fiable.
- Chaque migration est additive ou explicitement versionnée, avec sauvegarde et tests de restauration.

## Prototypes avant engagement

Collage direct et séquentiel, raccourcis globaux, capture d’écran et pipette : vérifier les capacités COSMIC/Wayland et définir les replis. La copie suivie de Ctrl+V reste disponible. Mise à jour « un clic » : choisir le canal Fedora et son mécanisme d’authentification avant de promettre une installation intégrée. Bandeau flottant, expansion textuelle et suppression locale du fond d’image : hors versions engagées tant que faisabilité, ressources et qualité ne sont pas validées.

Synchronisation, services distants et IA ne sont pas requis. Corrections, performances, accessibilité et documentation FR/EN font partie de chaque livraison.
