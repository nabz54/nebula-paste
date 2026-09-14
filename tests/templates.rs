use nebula_paste::{
    model::Clip,
    storage::Store,
    templates::{self, Archive, Conflict, Template},
};
use std::{collections::HashMap, os::unix::fs::PermissionsExt};
fn template(id: u32) -> Template {
    Template {
        id: format!("{id:032x}"),
        title: format!("Modèle {id}"),
        body: "Bonjour {{nom}}, serveur {{serveur}} : {{nom}}".into(),
        collection: "Travail".into(),
        created_at: 10,
        updated_at: 20,
    }
}
#[test]
fn literal_substitution_preserves_unicode_commands_and_repeated_values() {
    let body = "été {{nom}} / {{serveur}} / {{nom}} / {literal}";
    assert_eq!(templates::fields(body).unwrap(), vec!["nom", "serveur"]);
    let values = HashMap::from([
        ("nom".into(), "Zoë {{autre}}".into()),
        ("serveur".into(), "$(touch /tmp/never-execute); `id`".into()),
    ]);
    assert_eq!(
        templates::expand(body, &values).unwrap(),
        "été Zoë {{autre}} / $(touch /tmp/never-execute); `id` / Zoë {{autre}} / {literal}"
    );
    assert_eq!(
        templates::expand("  line\n", &HashMap::new()).unwrap(),
        "  line\n"
    );
}
#[test]
fn malformed_missing_and_excessive_fields_are_rejected() {
    for body in [
        "{{}}",
        "{{ nom }}",
        "{{nom",
        "nom}}",
        "{{{{nom}}",
        "{{x\ny}}",
        "a\0b",
    ] {
        assert!(templates::fields(body).is_err(), "{body}");
    }
    assert!(templates::expand("{{nom}}", &HashMap::new()).is_err());
    assert!(templates::expand("{{nom}}", &HashMap::from([("nom".into(), "  ".into())])).is_err());
    let many = (0..33)
        .map(|i| format!("{{{{field{i}}}}}"))
        .collect::<String>();
    assert!(templates::fields(&many).is_err());
    assert!(
        templates::expand(
            "{{a}}{{a}}",
            &HashMap::from([("a".into(), "x".repeat(templates::MAX_OUTPUT))])
        )
        .is_err()
    );
}
#[test]
fn crud_survives_reopen_and_history_retention_does_not_touch_templates() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("history.sqlite3");
    let id;
    {
        let mut s = Store::open(&path).unwrap();
        id = s
            .save_template(None, "Réponse", "  Bonjour {{nom}}\n", "Travail")
            .unwrap();
        let c = Clip::new("text/plain".into(), b"keep original".to_vec(), 1).unwrap();
        s.insert(&c).unwrap();
        s.pin(&c.id, true).unwrap();
        s.clear_unpinned().unwrap();
        s.purge_older_than(i64::MAX).unwrap();
        assert_eq!(s.load().unwrap()[0].bytes, c.bytes);
        assert_eq!(s.templates().unwrap()[0].body, "  Bonjour {{nom}}\n");
        s.save_template(Some(&id), "Édité", "Texte changé", "Autre")
            .unwrap();
        s.rename_collection("Autre", "Projet").unwrap();
        assert_eq!(s.templates().unwrap()[0].collection, "Projet");
        s.delete_collection("Projet").unwrap();
        assert_eq!(s.templates().unwrap()[0].collection, "");
    }
    let s = Store::open(&path).unwrap();
    assert_eq!(s.templates().unwrap()[0].title, "Édité");
    s.delete_template(&id).unwrap();
    assert!(s.templates().unwrap().is_empty());
    assert_eq!(s.load().unwrap().len(), 1);
    assert!(s.save_template(Some(&id), "x", "x", "").is_err());
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
    assert!(s.templates().unwrap().is_empty());
}
#[test]
fn json_roundtrip_is_private_and_does_not_overwrite() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("templates.json");
    let a = Archive::new(vec![template(1)]);
    a.export_new(&p).unwrap();
    assert_eq!(Archive::read(&p).unwrap().templates, a.templates);
    assert_eq!(
        std::fs::metadata(&p).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(Archive::new(vec![]).export_new(&p).is_err());
    assert_eq!(Archive::read(&p).unwrap().templates, a.templates);
    std::fs::write(&p, b"{\"version\":999}").unwrap();
    assert!(Archive::read(&p).is_err());
}
#[test]
fn each_conflict_policy_is_explicit_and_preserves_other_templates() {
    let s = Store::in_memory().unwrap();
    let a = Archive::new(vec![template(1)]);
    assert_eq!(s.import_templates(&a, Conflict::Skip).unwrap().added, 1);
    let mut changed = a.clone();
    changed.templates[0].body = "replacement".into();
    assert_eq!(
        s.import_templates(&changed, Conflict::Skip)
            .unwrap()
            .skipped,
        1
    );
    assert_ne!(s.templates().unwrap()[0].body, "replacement");
    assert_eq!(
        s.import_templates(&changed, Conflict::KeepBoth)
            .unwrap()
            .added,
        1
    );
    assert_eq!(s.templates().unwrap().len(), 2);
    assert_eq!(
        s.import_templates(&changed, Conflict::Replace)
            .unwrap()
            .replaced,
        1
    );
    assert_eq!(
        s.templates()
            .unwrap()
            .iter()
            .find(|t| t.id == template(1).id)
            .unwrap()
            .body,
        "replacement"
    );
    assert_eq!(
        s.templates()
            .unwrap()
            .iter()
            .find(|t| t.id == template(1).id)
            .unwrap()
            .created_at,
        10
    );
}
#[test]
fn invalid_imports_and_collection_overflow_are_atomic() {
    let s = Store::in_memory().unwrap();
    let mut a = Archive::new(vec![template(1), template(2)]);
    a.templates[1].body = "{{oops".into();
    assert!(s.import_templates(&a, Conflict::Replace).is_err());
    assert!(s.templates().unwrap().is_empty());
    assert!(s.collections().unwrap().is_empty());
    for i in 0..127 {
        s.create_collection(&format!("c{i}")).unwrap();
    }
    a.templates[1].body = "ok".into();
    a.templates[1].collection = "New collection".into();
    assert!(s.import_templates(&a, Conflict::Skip).is_err());
    assert!(s.templates().unwrap().is_empty());
    assert_eq!(s.collections().unwrap().len(), 127);
    let mut bad = Archive::new(vec![template(1), template(1)]);
    assert!(bad.validate().is_err());
    bad.templates.pop();
    bad.version = 2;
    assert!(bad.validate().is_err());
}
#[test]
fn template_count_limit_rolls_back_whole_import() {
    let s = Store::in_memory().unwrap();
    s.import_templates(
        &Archive::new((0..499).map(template).collect()),
        Conflict::Skip,
    )
    .unwrap();
    assert!(
        s.import_templates(
            &Archive::new(vec![template(600), template(601)]),
            Conflict::Skip
        )
        .is_err()
    );
    assert_eq!(s.templates().unwrap().len(), 499);
}
#[test]
fn search_uses_accents_words_and_collection() {
    let mut t = template(1);
    t.title = "Réponse à Élodie".into();
    t.body = "Vérifier le serveur".into();
    assert!(t.matches("elodie verifier", "Travail"));
    assert!(!t.matches("elodie", "Personnel"));
    assert!(!t.matches("absent", ""));
}
