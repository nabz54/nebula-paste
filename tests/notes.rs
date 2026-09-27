use nebula_paste::{
    model::Clip,
    notes::{self, Archive, Conflict, Note},
    storage::Store,
};
use std::os::unix::fs::PermissionsExt;
fn note(id: u32) -> Note {
    Note {
        id: format!("{id:032x}"),
        title: format!("Modèle {id}"),
        body: "Bonjour {{nom}}, serveur {{serveur}} : {{nom}}".into(),
        collection: "Travail".into(),
        created_at: 10,
        updated_at: 20,
    }
}
#[test]
fn crud_survives_reopen_and_history_retention_does_not_touch_notes() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("history.sqlite3");
    let id;
    {
        let mut s = Store::open(&path).unwrap();
        id = s
            .save_note(None, "Réponse", "  Bonjour {{nom}}\n", "Travail")
            .unwrap();
        let c = Clip::new("text/plain".into(), b"keep original".to_vec(), 1).unwrap();
        s.insert(&c).unwrap();
        s.pin(&c.id, true).unwrap();
        s.clear_unpinned().unwrap();
        s.purge_older_than(i64::MAX).unwrap();
        assert_eq!(s.load().unwrap()[0].bytes, c.bytes);
        assert_eq!(s.notes().unwrap()[0].body, "  Bonjour {{nom}}\n");
        s.save_note(Some(&id), "Édité", "Texte changé", "Autre")
            .unwrap();
        s.rename_collection("Autre", "Projet").unwrap();
        assert_eq!(s.notes().unwrap()[0].collection, "Projet");
        s.delete_collection("Projet").unwrap();
        assert_eq!(s.notes().unwrap()[0].collection, "");
    }
    let s = Store::open(&path).unwrap();
    assert_eq!(s.notes().unwrap()[0].title, "Édité");
    s.delete_note(&id).unwrap();
    assert!(s.notes().unwrap().is_empty());
    assert_eq!(s.load().unwrap().len(), 1);
    assert!(s.save_note(Some(&id), "x", "x", "").is_err());
}
#[test]
fn migration_from_07_preserves_clips_collections_and_ocr() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("old.db");
    let c = rusqlite::Connection::open(&p).unwrap();
    c.execute_batch("CREATE TABLE clips(id TEXT PRIMARY KEY,mime TEXT NOT NULL,bytes BLOB NOT NULL,timestamp INTEGER NOT NULL,pinned INTEGER NOT NULL DEFAULT 0,category TEXT NOT NULL DEFAULT ''); CREATE TABLE collections(name TEXT PRIMARY KEY NOT NULL); INSERT INTO collections VALUES('Legacy'); CREATE TABLE image_text(clip_id TEXT PRIMARY KEY REFERENCES clips(id) ON DELETE CASCADE,language TEXT NOT NULL,text TEXT NOT NULL);").unwrap();
    let clip = Clip::new("text/plain".into(), b"legacy".to_vec(), 1).unwrap();
    c.execute(
        "INSERT INTO clips VALUES(?1,?2,?3,1,1,'Legacy')",
        rusqlite::params![clip.id, clip.mime, clip.bytes],
    )
    .unwrap();
    drop(c);
    let s = Store::open(&p).unwrap();
    assert_eq!(s.load().unwrap()[0].bytes, b"legacy");
    assert!(s.load().unwrap()[0].pinned);
    assert_eq!(s.collections().unwrap(), vec!["Legacy"]);
    assert!(s.notes().unwrap().is_empty());
}
#[test]
fn json_roundtrip_is_private_and_does_not_overwrite() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("notes.json");
    let a = Archive::new(vec![note(1)]);
    a.export_new(&p).unwrap();
    assert_eq!(Archive::read(&p).unwrap().notes, a.notes);
    assert_eq!(
        std::fs::metadata(&p).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(Archive::new(vec![]).export_new(&p).is_err());
    assert_eq!(Archive::read(&p).unwrap().notes, a.notes);
    std::fs::write(&p, b"{\"version\":999}").unwrap();
    assert!(Archive::read(&p).is_err());
}
#[test]
fn each_conflict_policy_is_explicit_and_preserves_other_notes() {
    let s = Store::in_memory().unwrap();
    let a = Archive::new(vec![note(1)]);
    assert_eq!(s.import_notes(&a, Conflict::Skip).unwrap().added, 1);
    let mut changed = a.clone();
    changed.notes[0].body = "replacement".into();
    assert_eq!(s.import_notes(&changed, Conflict::Skip).unwrap().skipped, 1);
    assert_ne!(s.notes().unwrap()[0].body, "replacement");
    assert_eq!(
        s.import_notes(&changed, Conflict::KeepBoth).unwrap().added,
        1
    );
    assert_eq!(s.notes().unwrap().len(), 2);
    assert_eq!(
        s.import_notes(&changed, Conflict::Replace)
            .unwrap()
            .replaced,
        1
    );
    assert_eq!(
        s.notes()
            .unwrap()
            .iter()
            .find(|t| t.id == note(1).id)
            .unwrap()
            .body,
        "replacement"
    );
    assert_eq!(
        s.notes()
            .unwrap()
            .iter()
            .find(|t| t.id == note(1).id)
            .unwrap()
            .created_at,
        10
    );
}
#[test]
fn invalid_imports_and_collection_overflow_are_atomic() {
    let s = Store::in_memory().unwrap();
    let mut a = Archive::new(vec![note(1), note(2)]);
    a.notes[1].body = "{{oops".into();
    assert!(s.import_notes(&a, Conflict::Replace).is_err());
    assert!(s.notes().unwrap().is_empty());
    assert!(s.collections().unwrap().is_empty());
    for i in 0..127 {
        s.create_collection(&format!("c{i}")).unwrap();
    }
    a.notes[1].body = "ok".into();
    a.notes[1].collection = "New collection".into();
    assert!(s.import_notes(&a, Conflict::Skip).is_err());
    assert!(s.notes().unwrap().is_empty());
    assert_eq!(s.collections().unwrap().len(), 127);
    let mut bad = Archive::new(vec![note(1), note(1)]);
    assert!(bad.validate().is_err());
    bad.notes.pop();
    bad.version = 2;
    assert!(bad.validate().is_err());
}
#[test]
fn note_count_limit_rolls_back_whole_import() {
    let s = Store::in_memory().unwrap();
    s.import_notes(&Archive::new((0..499).map(note).collect()), Conflict::Skip)
        .unwrap();
    assert!(
        s.import_notes(&Archive::new(vec![note(600), note(601)]), Conflict::Skip)
            .is_err()
    );
    assert_eq!(s.notes().unwrap().len(), 499);
}
#[test]
fn search_uses_accents_words_and_collection() {
    let mut t = note(1);
    t.title = "Réponse à Élodie".into();
    t.body = "Vérifier le serveur".into();
    assert!(t.matches("elodie verifier", "Travail"));
    assert!(!t.matches("elodie", "Personnel"));
    assert!(!t.matches("absent", ""));
}
