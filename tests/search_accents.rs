use nebula_paste::model::{Clip, Kind};

#[test]
fn search_ignores_french_accents_case_and_whitespace_without_changing_bytes() {
    let original = "  Été à Nancy : déjà prêt, œuvre et café.\n";
    let clip = Clip::new("text/plain".into(), original.as_bytes().to_vec(), 1).unwrap();
    assert!(clip.matches("ETE  deja\nPRET oeuvre cafe", None, false, ""));
    assert!(!clip.matches("hiver", None, false, ""));
    assert_eq!(clip.bytes, original.as_bytes());
}

#[test]
fn decomposed_accents_and_non_latin_text_remain_searchable() {
    let clip = Clip::new(
        "text/plain".into(),
        "cafe\u{0301} 日本語 العربية".as_bytes().to_vec(),
        1,
    )
    .unwrap();
    assert!(clip.matches("café 日本語 العربية", None, false, ""));
    assert!(!clip.matches("日本語 absent", None, false, ""));
}

#[test]
fn ocr_and_collection_words_combine_without_weakening_filters() {
    let mut clip = Clip::new(
        "image/png".into(),
        include_bytes!("fixtures/ocr.png").to_vec(),
        1,
    )
    .unwrap();
    clip.pinned = true;
    clip.category = "École".into();
    assert!(clip.matches_with_ocr("ecole ete", Some(Kind::Image), true, "École", "Été"));
    assert!(!clip.matches_with_ocr("ete", Some(Kind::Text), true, "École", "Été"));
    assert!(!clip.matches_with_ocr("ete", None, true, "Autre", "Été"));
    assert_eq!(clip.category, "École");
}
