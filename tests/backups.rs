use nebula_paste::{
    backup::{Archive, Conflict},
    model::Clip,
    settings::Settings,
    storage::Store,
};
fn add(s: &mut Store, text: &str, time: i64) -> String {
    let c = Clip::new("text/plain".into(), text.as_bytes().to_vec(), time).unwrap();
    s.insert(&c).unwrap();
    c.id
}
#[test]
fn private_roundtrip_preserves_metadata_and_templates() {
    let mut s = Store::in_memory().unwrap();
    let id = add(&mut s, "Bonjour été", 42);
    s.pin(&id, true).unwrap();
    s.category(&id, "Travail").unwrap();
    s.create_collection("Vide").unwrap();
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("backup.json");
    s.history_archive().unwrap().export_new(&p).unwrap();
    let a = Archive::read(&p).unwrap();
    let mut t = Store::in_memory().unwrap();
    t.save_template(None, "model", "keep", "").unwrap();
    assert_eq!(
        t.restore_history(&a, Conflict::KeepLocal, true)
            .unwrap()
            .added,
        1
    );
    let c = t.load().unwrap().remove(0);
    assert_eq!(c.bytes, "Bonjour été".as_bytes());
    assert_eq!(c.timestamp, 42);
    assert!(c.pinned);
    assert_eq!(c.category, "Travail");
    assert!(t.collections().unwrap().contains(&"Vide".into()));
    assert_eq!(t.templates().unwrap().len(), 1);
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(&p).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(s.history_archive().unwrap().export_new(&p).is_err());
}
#[test]
fn conflicts_preserve_unrelated_entries() {
    let mut s = Store::in_memory().unwrap();
    let id = add(&mut s, "same", 1);
    s.pin(&id, true).unwrap();
    s.category(&id, "Backup").unwrap();
    let a = s.history_archive().unwrap().validate().unwrap();
    let mut t = Store::in_memory().unwrap();
    add(&mut t, "same", 9);
    add(&mut t, "other", 10);
    t.category(&id, "Local").unwrap();
    t.restore_history(&a, Conflict::KeepLocal, true).unwrap();
    let c = t.load().unwrap().into_iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.timestamp, 9);
    assert_eq!(c.category, "Local");
    assert!(!c.pinned);
    t.restore_history(&a, Conflict::UseBackup, true).unwrap();
    let c = t.load().unwrap().into_iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.timestamp, 1);
    assert_eq!(c.category, "Backup");
    assert!(c.pinned);
    assert_eq!(t.load().unwrap().len(), 2);
}
#[test]
fn malformed_archives_are_rejected() {
    let mut s = Store::in_memory().unwrap();
    add(&mut s, "test", 1);
    let mut a = s.history_archive().unwrap();
    a.clips[0].id = "bad".into();
    assert!(a.validate().is_err());
    let mut a = s.history_archive().unwrap();
    a.clips.push(a.clips[0].clone());
    assert!(a.validate().is_err());
    let mut a = s.history_archive().unwrap();
    a.version = 99;
    assert!(a.validate().is_err());
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("bad.json");
    std::fs::write(&p, b"{\"format\":").unwrap();
    assert!(Archive::read(&p).is_err());
}
#[test]
fn capacity_failure_rolls_back_metadata_and_collections() {
    let mut t = Store::in_memory().unwrap();
    for i in 0..500 {
        add(&mut t, &format!("entry {i}"), 10);
    }
    let mut s = Store::in_memory().unwrap();
    let id = add(&mut s, "entry 0", 1);
    s.pin(&id, true).unwrap();
    s.category(&id, "New").unwrap();
    add(&mut s, "overflow", 1);
    assert!(
        t.restore_history(
            &s.history_archive().unwrap().validate().unwrap(),
            Conflict::UseBackup,
            true
        )
        .is_err()
    );
    assert_eq!(t.load().unwrap().len(), 500);
    assert!(t.collections().unwrap().is_empty());
    let c = t.load().unwrap().into_iter().find(|c| c.id == id).unwrap();
    assert!(!c.pinned);
    assert_eq!(c.timestamp, 10);
}
#[test]
fn interrupted_restore_rolls_back_after_reopen() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("history.db");
    let mut t = Store::open(&p).unwrap();
    let id = add(&mut t, "existing", 10);
    let mut s = Store::in_memory().unwrap();
    add(&mut s, "first", 1);
    add(&mut s, "second", 2);
    s.create_collection("New").unwrap();
    let a = s.history_archive().unwrap().validate().unwrap();
    let c = rusqlite::Connection::open(&p).unwrap();
    c.execute_batch("CREATE TRIGGER fail_restore BEFORE INSERT ON clips WHEN (SELECT COUNT(*) FROM clips)>=2 BEGIN SELECT RAISE(ABORT,'simulated interruption'); END;").unwrap();
    drop(c);
    assert!(t.restore_history(&a, Conflict::KeepLocal, true).is_err());
    drop(t);
    let t = Store::open(&p).unwrap();
    assert_eq!(t.load().unwrap().len(), 1);
    assert_eq!(t.load().unwrap()[0].id, id);
    assert!(t.collections().unwrap().is_empty());
}
#[test]
fn diagnostic_contains_no_private_text() {
    let mut s = Store::in_memory().unwrap();
    let id = add(&mut s, "secret-password-XYZ", 1);
    s.category(&id, "PrivateClient").unwrap();
    s.save_template(None, "ConfidentialTitle", "secret-template", "")
        .unwrap();
    let report = s.diagnostic(&Settings::default()).unwrap();
    for secret in [
        &id,
        "secret-password-XYZ",
        "PrivateClient",
        "ConfidentialTitle",
        "secret-template",
    ] {
        assert!(!report.contains(secret));
    }
    let j: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert_eq!(j["clips"], 1);
    assert_eq!(j["templates"], 1);
}
#[test]
fn ocr_requires_enabled_image_search() {
    let mut s = Store::in_memory().unwrap();
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 2)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let c = Clip::new("image/png".into(), bytes.into_inner(), 1).unwrap();
    s.insert(&c).unwrap();
    s.index_image(&c.id, "eng", "OCR secret").unwrap();
    let a = s.history_archive().unwrap().validate().unwrap();
    let mut t = Store::in_memory().unwrap();
    t.restore_history(&a, Conflict::KeepLocal, false).unwrap();
    assert!(t.image_index("eng").unwrap().is_empty());
    t.restore_history(&a, Conflict::UseBackup, true).unwrap();
    assert_eq!(t.image_index("eng").unwrap()[&c.id], "OCR secret");
}
#[test]
fn background_snapshot_is_read_only_and_roundtrips() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("history.db");
    let mut s = Store::open(&p).unwrap();
    add(&mut s, "snapshot", 1);
    let bytes = Store::export_snapshot(&p, None).unwrap();
    let a: Archive = serde_json::from_slice(&bytes).unwrap();
    assert!(a.validate().is_ok());
    let missing = d.path().join("missing.db");
    assert!(Store::export_snapshot(&missing, None).is_err());
    assert!(!missing.exists());
}
