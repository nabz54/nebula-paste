use nebula_paste::{settings::Settings, text_actions::Preferences};
#[test]
fn action_preferences_persist_and_corrupt_sections_fall_back() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("prefs");
    let mut s = Settings::default();
    s.actions.bind(3, "Ctrl+Shift+K").unwrap();
    s.actions.slots[2] = Some("favorite-id".into());
    s.actions.clicks = [2, 1, 0];
    s.save(&path).unwrap();
    assert_eq!(Settings::load(&path), s);
    let bad = "actions = {\"bindings\":[],\"slots\":[null,null,null,null,null],\"clicks\":[0,0,99]}\nkeep-open = true\n";
    let parsed = Settings::parse(bad);
    assert_eq!(parsed.actions, Preferences::default());
    assert!(parsed.keep_open);
}
