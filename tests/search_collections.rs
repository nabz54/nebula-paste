use nebula_paste::{
    model::{Clip, Kind},
    settings::Settings,
    storage::{MAX_INDEX_CHARS, Store},
};

fn image(timestamp: i64) -> Clip {
    Clip::new(
        "image/png".into(),
        include_bytes!("fixtures/ocr.png").to_vec(),
        timestamp,
    )
    .unwrap()
}

#[test]
fn migrates_old_categories_and_preserves_empty_collections() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("history.sqlite3");
    let c = image(123);
    {
        let old = rusqlite::Connection::open(&path).unwrap();
        old.execute_batch("CREATE TABLE clips (id TEXT PRIMARY KEY, mime TEXT NOT NULL, bytes BLOB NOT NULL, timestamp INTEGER NOT NULL, pinned INTEGER NOT NULL DEFAULT 0, category TEXT NOT NULL DEFAULT '');").unwrap();
        old.execute(
            "INSERT INTO clips VALUES(?1,?2,?3,?4,1,'Travail')",
            rusqlite::params![c.id, c.mime, c.bytes, c.timestamp],
        )
        .unwrap();
    }
    let store = Store::open(&path).unwrap();
    assert_eq!(store.collections().unwrap(), ["Travail"]);
    store.create_collection(" Vide ").unwrap();
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(store.collections().unwrap(), ["Travail", "Vide"]);
    let restored = store.load().unwrap().remove(0);
    assert!(restored.pinned);
    assert_eq!(restored.bytes, c.bytes);
    assert_eq!(restored.timestamp, 123);
}

#[test]
fn collection_changes_are_atomic_and_never_delete_clips() {
    let mut store = Store::in_memory().unwrap();
    let c = image(1);
    store.insert(&c).unwrap();
    store.pin(&c.id, true).unwrap();
    store.category(&c.id, "Travail").unwrap();
    store.create_collection("Archives").unwrap();
    assert!(store.rename_collection("Travail", "Archives").is_err());
    assert_eq!(store.load().unwrap()[0].category, "Travail");
    store.rename_collection("Travail", "Projets").unwrap();
    assert_eq!(store.load().unwrap()[0].category, "Projets");
    store.index_image(&c.id, "eng", "COSMIC desktop").unwrap();
    store.delete_collection("Projets").unwrap();
    let kept = store.load().unwrap().remove(0);
    assert!(kept.category.is_empty());
    assert!(kept.pinned);
    assert_eq!(kept.bytes, c.bytes);
    assert_eq!(store.image_index("eng").unwrap()[&c.id], "COSMIC desktop");
    assert!(store.create_collection("  ").is_err());
    assert!(store.create_collection("x\ny").is_err());
    assert!(store.create_collection(&"é".repeat(41)).is_err());
    assert!(store.create_collection("Archives").is_err());
    assert!(store.category("missing", "Ghost").is_err());
    assert!(!store.collections().unwrap().contains(&"Ghost".into()));
}

#[test]
fn ocr_search_combines_filters_without_rewriting_the_clip() {
    let mut store = Store::in_memory().unwrap();
    let original = image(1);
    store.insert(&original).unwrap();
    store.category(&original.id, "Factures").unwrap();
    store.pin(&original.id, true).unwrap();
    assert!(
        store
            .index_image(&original.id, "fra+eng", "Total Septembre 42 euros")
            .unwrap()
    );
    let c = store.load().unwrap().remove(0);
    let index = store.image_index("fra+eng").unwrap();
    let text = &index[&c.id];
    assert!(c.matches_with_ocr(
        "FACTURES septembre 42",
        Some(Kind::Image),
        true,
        "Factures",
        text
    ));
    assert!(!c.matches_with_ocr("octobre", None, false, "", text));
    assert!(!c.matches_with_ocr("42", Some(Kind::Text), false, "", text));
    assert_eq!(c.bytes, original.bytes);
    assert_eq!(c.text, original.text);
    assert_eq!(store.load().unwrap().len(), 1);
    assert!(store.image_index("eng").unwrap().is_empty());
    store.clear_image_index().unwrap();
    assert!(store.image_index("fra+eng").unwrap().is_empty());
    assert_eq!(store.load().unwrap().len(), 1);
}

#[test]
fn index_is_bounded_and_follows_source_deletion_and_retention() {
    let mut store = Store::in_memory().unwrap();
    let c = image(1);
    store.insert(&c).unwrap();
    store
        .index_image(&c.id, "eng", &"é".repeat(MAX_INDEX_CHARS + 10))
        .unwrap();
    assert_eq!(
        store.image_index("eng").unwrap()[&c.id].chars().count(),
        MAX_INDEX_CHARS
    );
    assert!(!store.index_image("missing", "eng", "orphan").unwrap());
    let text = Clip::new("text/plain".into(), b"hello".to_vec(), 1).unwrap();
    store.insert(&text).unwrap();
    assert!(!store.index_image(&text.id, "eng", "wrong type").unwrap());
    assert!(
        store
            .index_image(&c.id, "unknown", "invalid language")
            .is_err()
    );
    store.pin(&c.id, true).unwrap();
    store.clear_unpinned().unwrap();
    assert_eq!(store.image_index("eng").unwrap().len(), 1);
    store.purge_older_than(2).unwrap();
    assert_eq!(store.image_index("eng").unwrap().len(), 1);
    store.pin(&c.id, false).unwrap();
    store.purge_older_than(2).unwrap();
    assert!(store.image_index("eng").unwrap().is_empty());
    store.insert(&c).unwrap();
    store.index_image(&c.id, "eng", "new").unwrap();
    store.delete(&c.id).unwrap();
    assert!(store.image_index("eng").unwrap().is_empty());
}

#[test]
fn indexing_is_opt_in_and_setting_survives_restart() {
    assert!(!Settings::default().ocr_indexing);
    assert!(!Settings::parse("ocr-indexing = invalid").ocr_indexing);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.conf");
    let settings = Settings::parse("ocr-indexing = true\nocr-language = eng");
    settings.save(&path).unwrap();
    assert_eq!(Settings::load(&path), settings);
}
