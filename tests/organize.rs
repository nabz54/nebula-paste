use nebula_paste::{
    model::{Clip, Kind},
    organize::{Policy, Rule},
    storage::Store,
};
fn sample(s: &mut Store, text: &str, time: i64) -> Clip {
    let c = Clip::new("text/plain".into(), text.as_bytes().to_vec(), time).unwrap();
    s.insert(&c).unwrap();
    c
}
#[test]
fn boards_preserve_items_through_rename_and_column_deletion() {
    let mut s = Store::in_memory().unwrap();
    s.create_collection("Work").unwrap();
    let c = sample(&mut s, "item", 1);
    s.category(&c.id, "Work").unwrap();
    s.save_column("Work", None, "Doing").unwrap();
    s.save_column("Work", None, "Done").unwrap();
    let cols = s.board_columns("Work").unwrap();
    s.board_move("Work", 0, &c.id, Some(cols[0].id)).unwrap();
    s.order_column("Work", cols[1].id, true).unwrap();
    assert_eq!(s.board_columns("Work").unwrap()[0].name, "Done");
    s.rename_collection("Work", "Renamed").unwrap();
    assert_eq!(s.board_assignments("Renamed").unwrap().len(), 1);
    s.delete_column(cols[0].id).unwrap();
    assert!(s.board_assignments("Renamed").unwrap().is_empty());
    assert_eq!(s.load().unwrap().len(), 1);
    s.delete_collection("Renamed").unwrap();
    assert!(s.board_columns("Renamed").unwrap().is_empty());
    assert_eq!(s.load().unwrap()[0].category, "");
}
#[test]
fn board_rejects_foreign_columns_and_survives_reopening() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("history.db");
    let mut s = Store::open(&p).unwrap();
    s.create_collection("A").unwrap();
    s.create_collection("B").unwrap();
    let c = sample(&mut s, "sample", 1);
    s.category(&c.id, "A").unwrap();
    s.save_column("B", None, "Wrong").unwrap();
    let id = s.board_columns("B").unwrap()[0].id;
    assert!(s.board_move("A", 0, &c.id, Some(id)).is_err());
    s.save_column("A", None, "Right").unwrap();
    let id = s.board_columns("A").unwrap()[0].id;
    s.board_move("A", 0, &c.id, Some(id)).unwrap();
    s.set_board_enabled("A", true).unwrap();
    drop(s);
    let s = Store::open(&p).unwrap();
    assert!(s.board_enabled("A"));
    assert_eq!(s.board_assignments("A").unwrap()[&(0, c.id)], id);
}
#[test]
fn rule_preview_conflicts_staleness_and_undo() {
    let mut s = Store::in_memory().unwrap();
    s.create_collection("Dev").unwrap();
    s.create_collection("Other").unwrap();
    let c = sample(&mut s, "Élodie github.com", 1);
    let r = Rule {
        id: 0,
        kind: Some(Kind::Text),
        contains: "elodie".into(),
        destination: "Dev".into(),
        enabled: true,
    };
    s.save_rule(&r).unwrap();
    let p = s.rule_preview().unwrap();
    assert_eq!(p.len(), 1);
    let u = s.apply_rules(&p).unwrap();
    assert_eq!(s.load().unwrap()[0].category, "Dev");
    s.undo_rules(&u).unwrap();
    assert_eq!(s.load().unwrap()[0].category, "");
    s.save_rule(&Rule {
        destination: "Other".into(),
        ..r
    })
    .unwrap();
    assert!(s.apply_rules(&p).is_err());
    assert_eq!(s.rule_preview().unwrap()[0].destinations.len(), 2);
    s.classify_new(&c.id).unwrap();
    assert_eq!(s.load().unwrap()[0].category, "");
    s.pin(&c.id, true).unwrap();
    assert!(s.rule_preview().unwrap().is_empty());
}
#[test]
fn expiration_protects_favorites_notes_exclusions_and_recent_use() {
    let mut s = Store::in_memory().unwrap();
    s.create_collection("Keep").unwrap();
    let old = sample(&mut s, "old", 1);
    let used = sample(&mut s, "used", 1);
    let pinned = sample(&mut s, "pinned", 1);
    let excluded = sample(&mut s, "excluded", 1);
    s.pin(&pinned.id, true).unwrap();
    s.category(&excluded.id, "Keep").unwrap();
    s.save_note(None, "Note", "persist", "").unwrap();
    s.save_template(None, "Template", "persist", "").unwrap();
    s.mark_used(&used.id, 200000).unwrap();
    let p = Policy {
        days: 1,
        excluded: vec!["Keep".into()],
    };
    assert_eq!(
        s.unused_preview(&p, 200000).unwrap(),
        vec![(old.id, String::new())]
    );
    assert_eq!(s.expire_unused(200000).unwrap(), 0);
    s.save_unused_policy(&p).unwrap();
    assert_eq!(s.expire_unused(200000).unwrap(), 1);
    assert_eq!(s.load().unwrap().len(), 3);
    assert_eq!(s.notes().unwrap().len(), 1);
    assert_eq!(s.templates().unwrap().len(), 1);
}
#[test]
fn migration_backs_up_legacy_database_once() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("history.db");
    let s = Store::open(&p).unwrap();
    drop(s);
    let c = rusqlite::Connection::open(&p).unwrap();
    c.execute_batch("DROP TABLE board_columns;").unwrap();
    drop(c);
    let s = Store::open(&p).unwrap();
    assert!(p.with_extension("pre-1.5.sqlite3").exists());
    drop(s);
    assert!(Store::open(&p).is_ok());
}
