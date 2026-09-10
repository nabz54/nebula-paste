use nebula_paste::{
    clipboard::select_mime,
    model::{Clip, Kind, MAX_ITEMS, color},
    storage::Store,
};

fn clip(text: &str, timestamp: i64) -> Clip {
    Clip::new(
        "text/plain;charset=utf-8".into(),
        text.as_bytes().to_vec(),
        timestamp,
    )
    .unwrap()
}

#[test]
fn duplicates_keep_favorite_and_category_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("private/history.sqlite3");
    let mut db = Store::open(&path).unwrap();
    let a = clip("bonjour", 1);
    db.insert(&a).unwrap();
    db.pin(&a.id, true).unwrap();
    db.category(&a.id, "Travail").unwrap();
    db.insert(&clip("bonjour", 2)).unwrap();
    drop(db);
    let entries = Store::open(&path).unwrap().load().unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].pinned);
    assert_eq!(entries[0].category, "Travail");
    assert_eq!(entries[0].timestamp, 2);
}

#[test]
fn retention_preserves_favorites_and_evicts_oldest() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Store::open(&dir.path().join("data/history.sqlite3")).unwrap();
    let pinned = clip("keep", 0);
    db.insert(&pinned).unwrap();
    db.pin(&pinned.id, true).unwrap();
    for i in 1..=MAX_ITEMS {
        db.insert(&clip(&format!("entry-{i}"), i as i64)).unwrap();
    }
    let entries = db.load().unwrap();
    assert_eq!(entries.len(), MAX_ITEMS);
    assert!(entries.iter().any(|e| e.id == pinned.id));
    assert!(!entries.iter().any(|e| e.text == "entry-1"));
    db.clear_unpinned().unwrap();
    assert_eq!(db.load().unwrap().len(), 1);
}

#[test]
fn full_favorites_roll_back_insertion() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Store::open(&dir.path().join("data/history.sqlite3")).unwrap();
    for i in 0..MAX_ITEMS {
        let c = clip(&format!("pinned-{i}"), i as i64);
        db.insert(&c).unwrap();
        db.pin(&c.id, true).unwrap();
    }
    assert!(db.insert(&clip("overflow", 999)).is_err());
    let entries = db.load().unwrap();
    assert_eq!(entries.len(), MAX_ITEMS);
    assert!(entries.iter().all(|e| e.pinned));
}

#[test]
fn utf8_whitespace_and_original_bytes_survive() {
    let text = "  é العربية 日本語\n\t  ";
    let c = clip(text, 1);
    assert_eq!(c.bytes, text.as_bytes());
    assert_eq!(c.text, text);
    assert_eq!(color("#ééé"), None);
    assert_eq!(color("#abc"), Some([170, 187, 204]));
}

#[test]
fn mime_filter_excludes_marked_secrets_and_prefers_files() {
    let list = |values: &[&str]| values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
    assert_eq!(
        select_mime(&list(&["text/plain", "x-kde-passwordManagerHint"])),
        None
    );
    assert_eq!(
        select_mime(&list(&["text/plain", "image/png"])),
        Some("image/png".into())
    );
    assert_eq!(
        select_mime(&list(&["text/plain", "text/uri-list"])),
        Some("text/uri-list".into())
    );
    assert_eq!(select_mime(&list(&["application/octet-stream"])), None);
}

#[test]
fn categorization_and_combined_search() {
    assert_eq!(clip("https://example.org", 1).kind, Kind::Link);
    assert_eq!(clip("sudo systemctl status httpd", 1).kind, Kind::Code);
    let mut c = clip("sudo systemctl status httpd", 1);
    c.pinned = true;
    c.category = "Linux".into();
    assert!(c.matches("HTTPD linux", Some(Kind::Code), true, "Linux"));
    assert!(!c.matches("nginx", None, false, ""));
    assert!(!c.matches("", Some(Kind::Image), false, ""));
}

#[test]
fn invalid_images_and_oversized_clips_are_rejected() {
    assert!(Clip::new("image/png".into(), b"not png".to_vec(), 0).is_err());
    assert!(
        Clip::new(
            "text/plain".into(),
            vec![b'x'; nebula_paste::model::MAX_CLIP_BYTES + 1],
            0
        )
        .is_err()
    );
}

#[test]
fn plain_text_preserves_markup_code_decodes_local_uris_and_keeps_whitespace() {
    use nebula_paste::model::plain_text;
    let markup = Clip::new(
        "text/plain;charset=utf-8".into(),
        b"<p>Bonjour <b>COSMIC</b></p><p>&amp; la suite</p>".to_vec(),
        1,
    )
    .unwrap();
    assert_eq!(plain_text(&markup).unwrap(), markup.text);
    let files = Clip::new(
        "text/uri-list".into(),
        "file:///tmp/mon%20fichier.txt\r\n#commentaire\r\nfile:///tmp/été%.txt\r\n"
            .as_bytes()
            .to_vec(),
        1,
    )
    .unwrap();
    assert_eq!(
        plain_text(&files).unwrap(),
        "/tmp/mon fichier.txt\n/tmp/été%.txt"
    );
    // Un texte sans mise en forme traverse la conversion sans être réécrit.
    assert_eq!(
        plain_text(&clip("  été\tà Nancy\n", 1)).unwrap(),
        "  été\tà Nancy\n"
    );
    let image = Clip::new(
        "image/png".into(),
        include_bytes!("fixtures/ocr.png").to_vec(),
        1,
    )
    .unwrap();
    assert!(plain_text(&image).is_none());
}

#[test]
fn retention_removes_old_entries_and_spares_favorites() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Store::open(&dir.path().join("data/history.sqlite3")).unwrap();
    let mut old = clip("ancien", 1_000);
    let kept = clip("ancien favori", 1_001);
    let recent = clip("récent", 9_000);
    for entry in [&old, &kept, &recent] {
        db.insert(entry).unwrap();
    }
    db.pin(&kept.id, true).unwrap();
    assert_eq!(db.purge_older_than(5_000).unwrap(), 1);
    let entries = db.load().unwrap();
    assert_eq!(entries.len(), 2);
    assert!(!entries.iter().any(|e| e.id == old.id));
    // La restauration rend l’entrée avec son horodatage, sans doublon.
    old.category = "Travail".into();
    db.restore(&old).unwrap();
    old.category = "Travail".into();
    db.restore(&old).unwrap();
    let entries = db.load().unwrap();
    assert_eq!(entries.len(), 3);
    let restored = entries.iter().find(|e| e.id == old.id).unwrap();
    assert_eq!(restored.timestamp, 1_000);
    assert_eq!(restored.category, "Travail");
}

#[test]
fn storage_permissions_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data/history.sqlite3");
    let _db = Store::open(&path).unwrap();
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        std::fs::metadata(path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
}

#[test]
fn undo_restores_metadata_without_overwriting_a_recaptured_clip() {
    let mut db = Store::in_memory().unwrap();
    let mut original = clip("à restaurer", 123);
    original.pinned = true;
    original.category = "Travail".into();
    assert!(db.restore(&original).unwrap());
    let restored = db.load().unwrap().remove(0);
    assert!(restored.pinned);
    assert_eq!(restored.category, "Travail");
    assert_eq!(restored.timestamp, 123);
    db.delete(&original.id).unwrap();
    db.insert(&clip("à restaurer", 456)).unwrap();
    assert!(!db.restore(&original).unwrap());
    let recaptured = db.load().unwrap().remove(0);
    assert!(!recaptured.pinned);
    assert_eq!(recaptured.timestamp, 456);
}

#[test]
fn undo_at_capacity_does_not_evict_other_clips() {
    let mut db = Store::in_memory().unwrap();
    for i in 0..MAX_ITEMS {
        db.insert(&clip(&format!("item {i}"), i as i64)).unwrap();
    }
    let before: Vec<_> = db.load().unwrap().into_iter().map(|c| c.id).collect();
    assert!(db.restore(&clip("ancienne copie", 0)).is_err());
    let after: Vec<_> = db.load().unwrap().into_iter().map(|c| c.id).collect();
    assert_eq!(before, after);
}
