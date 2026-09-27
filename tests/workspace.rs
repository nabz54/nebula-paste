use nebula_paste::{model::Clip, notes, storage::Store, workspace::Key};
use std::collections::{HashMap, HashSet};
fn clip(store: &mut Store) -> Clip {
    let c = Clip::new("text/plain".into(), b"original".to_vec(), 100).unwrap();
    store.insert(&c).unwrap();
    c
}
#[test]
fn notes_are_literal_and_survive_history_clear() {
    let mut s = Store::in_memory().unwrap();
    clip(&mut s);
    let text = "  {{ unfinished $(id) `echo x`\n";
    s.save_note(None, "Literal", text, "").unwrap();
    assert_eq!(notes::expand(text, &HashMap::new()).unwrap(), text);
    s.clear_unpinned().unwrap();
    s.purge_older_than(i64::MAX).unwrap();
    assert!(s.load().unwrap().is_empty());
    assert_eq!(s.notes().unwrap()[0].body, text);
}
#[test]
fn mixed_selection_moves_and_undoes_atomically() {
    let mut s = Store::in_memory().unwrap();
    let c = clip(&mut s);
    let n = s.save_note(None, "Note", "body", "Before").unwrap();
    s.create_collection("After").unwrap();
    let keys = HashSet::from([Key::Clip(c.id.clone()), Key::Note(n)]);
    let undo = s.move_items(&keys, "After").unwrap();
    assert_eq!(s.load().unwrap()[0].category, "After");
    assert_eq!(s.notes().unwrap()[0].collection, "After");
    s.undo_items(&undo).unwrap();
    assert_eq!(s.load().unwrap()[0].category, "");
    assert_eq!(s.notes().unwrap()[0].collection, "Before");
    let mut invalid = keys;
    invalid.insert(Key::Note("missing".into()));
    assert!(s.move_items(&invalid, "After").is_err());
    assert!(s.delete_items(&invalid).is_err());
    assert_eq!(s.load().unwrap().len(), 1);
    assert_eq!(s.notes().unwrap()[0].collection, "Before");
}
#[test]
fn deletion_restores_favorites_and_notes_but_never_overwrites_a_new_copy() {
    let mut s = Store::in_memory().unwrap();
    let c = clip(&mut s);
    s.pin(&c.id, true).unwrap();
    let n = s.save_note(None, "Note", "body", "Old").unwrap();
    let keys = HashSet::from([Key::Clip(c.id.clone()), Key::Note(n.clone())]);
    let undo = s.delete_items(&keys).unwrap();
    assert!(s.notes().unwrap().is_empty());
    assert!(s.load().unwrap().is_empty());
    s.delete_collection("Old").unwrap();
    s.undo_items(&undo).unwrap();
    assert!(s.load().unwrap()[0].pinned);
    assert_eq!(s.notes().unwrap()[0].id, n);
    assert_eq!(s.notes().unwrap()[0].collection, "");
    let undo = s.delete_items(&keys).unwrap();
    s.insert(&c).unwrap();
    assert!(s.undo_items(&undo).is_err());
    assert!(s.notes().unwrap().is_empty());
}
#[test]
fn view_and_order_survive_reopen_and_rename() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("db");
    {
        let s = Store::open(&p).unwrap();
        s.create_collection("A").unwrap();
        s.create_collection("B").unwrap();
        s.set_collection_cards("B", false).unwrap();
        s.move_collection("B", true).unwrap();
        s.rename_collection("B", "Z").unwrap();
    }
    let s = Store::open(&p).unwrap();
    assert_eq!(s.collections().unwrap(), vec!["Z", "A"]);
    assert!(!s.collection_cards("Z"));
    assert!(s.collection_cards("A"));
}
#[test]
fn undo_move_refuses_changed_items_without_partial_changes() {
    let mut s = Store::in_memory().unwrap();
    let c = clip(&mut s);
    let n = s.save_note(None, "Note", "body", "A").unwrap();
    s.create_collection("B").unwrap();
    let keys = HashSet::from([Key::Clip(c.id.clone()), Key::Note(n.clone())]);
    let undo = s.move_items(&keys, "B").unwrap();
    s.save_note(Some(&n), "Note", "new body", "A").unwrap();
    assert!(s.undo_items(&undo).is_err());
    assert_eq!(s.load().unwrap()[0].category, "B");
    assert_eq!(s.notes().unwrap()[0].body, "new body");
}
