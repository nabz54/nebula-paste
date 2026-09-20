# Roadmap Nebula Paste

[English](ROADMAP.en.md)

Les versions ci-dessous sont des objectifs, pas des fonctionnalités déjà livrées ni des dates promises. La 0.7 reste une bêta ; sa validation sur Fedora COSMIC continue avant une version stable.

## 0.8 — Identité et modèles réutilisables

**Implémenté en 0.8.0-beta.1 :** identité 2A, modèles persistants, édition, champs littéraux, recherche et import/export avec gestion des conflits. Les essais interactifs Fedora COSMIC restent à effectuer avant une version stable.

**Fonctions livrées :**

1. Stockage des modèles séparé des copies : identifiant stable, titre, corps, collection facultative, dates de création/modification. Migration additive ; rétention et vidage de l’historique sans effet sur les modèles.
2. Vue Modèles accessible depuis le popup et la fenêtre complète : créer, modifier, rechercher dans titre/corps, classer, supprimer avec confirmation. Création depuis une copie texte sans modifier l’original.
3. Champs explicites `{{nom}}`, `{{date}}`, `{{serveur}}`, `{{ip}}` : formulaire avant copie, même valeur pour chaque occurrence, aperçu final, valeurs non enregistrées par défaut. `date` reste un champ à remplir pour cette première version. Aucune expansion de shell ni exécution de commande.
4. Import/export JSON versionné : modèles uniquement, limites de taille et de nombre, validation complète avant écriture, aperçu des conflits, aucune substitution silencieuse. Sauvegarde atomique à l’export et transaction à l’import.
5. Libellés, aide, changelog et guide de test en français et anglais.

**Critères de sortie :** tests de migration et conservation de l’historique ; CRUD persistant ; champs répétés/manquants/Unicode et syntaxe incorrecte ; aller-retour import/export et échec sans import partiel ; recherche accentuée ; navigation clavier et rendu compact/élargi sur COSMIC. La nouvelle identité doit rester lisible avec les thèmes et tailles de panneau utilisés.

## 0.9 — Fiabilité et maîtrise des données

**Implémenté dans la branche 0.9 bêta :** sauvegarde/restauration, diagnostic, impact de rétention, transparence, redimensionnement, isolation clavier et mesures reproductibles. Les critères de validation COSMIC réelle restent à vérifier.

**Proposé :** consolider placement, redimensionnement, focus clavier et changements de thème ; mesurer la recherche et l’OCR sur un historique rempli ; sauvegarder/restaurer l’historique avec aperçu et gestion des conflits ; clarifier les réglages de rétention ; exporter un diagnostic excluant le contenu des copies et des modèles.

**Critères de sortie :** tests de restauration interrompue ou invalide, conservation des favoris et collections, mesures reproductibles, matrice Fedora/COSMIC documentée, aucun blocage connu sur les parcours principaux.

## 1.0 — Version stable

**Proposé :** installation et mise à jour RPM reproductibles, migration préservant les données, documentation FR/EN complète, dépannage et désinstallation documentés, versions Fedora/COSMIC effectivement testées et annoncées.

**Critères de sortie :** plusieurs semaines d’usage réel, aucun défaut bloquant connu, installation neuve et mise à niveau vérifiées, limitations publiées. La date dépend des essais, pas d’une échéance arbitraire.

## Après 1.0

Synchronisation et extensions éventuelles : à décider séparément. Aucun service distant n’est requis par cette roadmap.
