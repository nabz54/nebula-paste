# Nebula Paste — Capture cosmique (2A)

[English](DESIGN-2A.en.md) · [Planche SVG](identity-2a.svg) · [Roadmap](ROADMAP.md)

Direction choisie : une nébuleuse asymétrique, une ouverture stellaire et deux angles de capture séparés. Le nuage porte « Nebula » ; les angles évoquent la capture des contenus du presse-papiers. Ils ne représentent pas une nouvelle fonction de capture d’écran.

## Fichiers intégrés

- `resources/io.github.nebulapaste.NebulaPaste.svg` : lanceur et en-tête, violet profond, lavande et cyan, aplats sans halo.
- `resources/io.github.nebulapaste.NebulaPaste-symbolic.svg` : panneau, une seule couleur recolorée par le thème. L’espace central est transparent, jamais peint en blanc.
- `docs/identity-2a.svg` : aperçu des vrais SVG à 16, 24 et 32 px, sur fonds clair et sombre. Afficher à 100 % pour juger les tailles natives.

Les deux versions partagent la même géométrie sur une grille de 24 unités, avec une marge de 2 unités. Les détails de couleur ne sont pas nécessaires à la reconnaissance. Le bouton du panneau conserve le dimensionnement et les interactions de libcosmic ; la pause conserve son indicateur existant.

Le script d’installation et le RPM utilisent déjà ces deux noms de fichiers. Le lanceur, l’en-tête incorporé au binaire et le panneau récupèrent ainsi la nouvelle identité après reconstruction et installation. Aucun nouvel asset externe ou téléchargement au démarrage.

## Thème et validation

L’identité ne fixe aucune couleur de surface, de sélection ou de texte de l’interface. Ces couleurs restent celles du système COSMIC. Aucun changement de données ou de préférences.

Les deux SVG ont été rendus et inspectés à 16, 24 et 32 px sur fond clair et sombre. Le rendu à 16 px conserve moins de détail ; la forme et les angles restent prioritaires. Cela ne remplace pas le test dans une session COSMIC : vérifier les tailles du panneau, les facteurs d’échelle, les deux thèmes et la pause après installation. Les captures des anciennes versions ne sont pas des captures de cette identité.
