# Tester Nebula Paste 0.9 bêta

[English](TESTING-0.9.en.md)

## Sauvegarde et restauration

Préférences → **Sauvegarde et diagnostic**. Inclus : octets des copies, dates, favoris, collections même vides et index OCR existant. Exclus : modèles, réglages et fichiers désignés par les URI (seul le lien est conservé). Les modèles gardent leur export séparé. L’archive JSON est privée (permissions 0600), mais non chiffrée. L’export vers un fichier existant est refusé, y compris après sélection du nom.

1. Créer des textes, une image indexée, un favori et une collection vide. Exporter puis modifier le classement et le statut favori d’une copie.
2. Importer : l’aperçu indique ajouts, conflits, favoris, collections et volume. Aucune écriture avant choix explicite et confirmation.
3. Un conflit désigne le même contenu (identifiant SHA-256). **Conserver les données locales** garde sa date, son classement, son statut favori et son index. **Utiliser les données sauvegardées** reprend ces métadonnées et l’index de l’archive. Les autres copies restent présentes ; les nouveaux éléments sont ajoutés dans les deux cas.
4. La confirmation désactive explicitement la limite d’âge, enregistrée avant la transaction : une ancienne copie restaurée ne doit pas être supprimée au rafraîchissement ou au redémarrage. Réappliquer ensuite la rétention choisie avec l’aperçu du nombre de copies concernées. Favoris et modèles sont protégés de la rétention.
5. Si la recherche d’images est désactivée, l’index OCR importé est ignoré. Réactiver la recherche reconstruit l’index localement.
6. Essayer JSON tronqué, version inconnue, empreinte incohérente et dépassement de capacité : aucune restauration partielle. Limites : 500 copies, 128 Mio de contenu, 128 collections et 256 Mio pour le JSON. La restauration n’évince jamais d’autres copies pour faire de la place.
7. Échap annule l’aperçu. Les raccourcis de copie d’historique ne doivent pas agir dans les préférences, les modèles ou les collections.

Le diagnostic contient uniquement version, système générique, architecture, compteurs et réglages OCR/rétention. Aucun contenu, titre, nom de collection, chemin, identifiant de copie ou variable d’environnement.

## Validation et mesures

```bash
cargo fmt --check
cargo test --locked
cargo build --locked
bash scripts/check-embedded-ocr.sh target/debug/nebula-paste
bash scripts/benchmark.sh
```

Le benchmark utilise seulement des données synthétiques : 1/100/500 copies et 200 requêtes sans accents. OCR : fixtures du dépôt, trois processus par langue, démarrage du moteur inclus. Profil de développement ; chiffres dépendants du CPU et de la charge, sans mesure de latence du compositeur.

| Essai réel | Fedora 44 + COSMIC |
|---|---|
| Installation, mise à niveau et relance | À tester |
| Popup compact/élargi, fenêtre séparée redimensionnée | À tester |
| Bords de panneau, plusieurs écrans, échelle 100/150/200 % | À tester |
| Thèmes clair/sombre, transparence activée/désactivée | À tester |
| Portail, annulation, clavier et focus | À tester |
| Restauration d’un historique rempli puis redémarrage | À tester |

Les tests automatisés ne prouvent pas le comportement d’une session COSMIC réelle. Cette version reste une bêta.
