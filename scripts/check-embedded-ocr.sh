#!/usr/bin/env bash
# Run after cargo build; neither PATH nor TESSDATA_PREFIX supplies OCR tools/data.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
binary="$(realpath -- "${1:-target/debug/nebula-paste}")"
ocr_check_dir="$(mktemp -d)"
trap 'rm -rf -- "$ocr_check_dir"' EXIT
mkdir "$ocr_check_dir/bin" "$ocr_check_dir/data"
dependencies="$(readelf -d -- "$binary")"
if [[ "$dependencies" == *libtesseract* || "$dependencies" == *liblept* ]]; then
    echo 'Échec : dépendance OCR dynamique détectée.' >&2
    exit 1
fi
for language in eng fra fra+eng; do
    fixture=tests/fixtures/ocr-french.png
    if [[ "$language" == eng ]]; then fixture=tests/fixtures/ocr.png; fi
    result="$(PATH="$ocr_check_dir/bin" TESSDATA_PREFIX="$ocr_check_dir/data" "$binary" --ocr "$fixture" "$language")"
    if [[ "$language" == eng ]]; then
        [[ "$result" == *'NEBULA PASTE 12345'* ]]
    else
        [[ "$result" == *'Été'* && "$result" == *'déjà'* && "$result" == *'élève'* && "$result" == *'café'* ]]
    fi
    printf '%s : %s\n' "$language" "$result"
done
echo 'OCR embarqué validé : PATH vide, modèles système ignorés, aucune dépendance dynamique Tesseract/Leptonica.'
