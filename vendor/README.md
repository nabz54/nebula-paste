# Composants OCR fournis

- Tesseract 5.5.1 : https://github.com/tesseract-ocr/tesseract/tree/5.5.1 — Apache-2.0.
- Leptonica 1.85.0 : https://github.com/DanBloomberg/leptonica/tree/1.85.0 — BSD, texte dans licenses/.
- Modèles eng et fra, tessdata_fast 4.1.0 : https://github.com/tesseract-ocr/tessdata_fast/tree/4.1.0 — Apache-2.0.

Archives sources originales : https://codeload.github.com/tesseract-ocr/tesseract/tar.gz/refs/tags/5.5.1 et https://codeload.github.com/DanBloomberg/leptonica/tar.gz/refs/tags/1.85.0.

Les fichiers sources et modèles sont inchangés. Pour permettre le transfert via la connexion GitHub, l’archive Leptonica est répartie en deux fichiers `.part1` et `.part2`. `build.rs` les concatène automatiquement dans le dossier de compilation avant extraction, sans outil supplémentaire ni téléchargement. L’archive reconstituée est identique à l’original (SHA-256 : `c01376bce0379d4ea4bc2ec5d5cbddaa49e2f06f88242619ab8c059e21adf233`).

SHA256SUMS contient les empreintes des parties, de l’archive Tesseract et des modèles ; il est vérifié à la compilation. Les licences sont conservées dans licenses/ ainsi que dans les archives sources.

Configuration : bibliothèques statiques, moteur LSTM uniquement, sans curl, libarchive, OpenMP, interface graphique Tesseract, outils d’entraînement ni optimisation spécifique au CPU hôte. Les codecs Leptonica sont désactivés : les pixels sont décodés par la bibliothèque image de Rust et transmis directement. CMake ne télécharge aucune dépendance native.

La bibliothèque standard C++ et les bibliothèques de base du système restent dynamiques, comme les dépendances graphiques COSMIC. Il ne s’agit pas d’un exécutable Linux entièrement statique.

Les options natives TESSERACT_DISABLE_DEBUG_FONTS et TESSERACT_IMAGEDATA_AS_PIX désactivent les polices réservées aux images de débogage et conservent les images intermédiaires en pixels. Elles évitent de solliciter les codecs Leptonica désactivés. Les erreurs natives restent visibles ; aucune redirection globale de stderr ni suppression des diagnostics n’est appliquée.
