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
