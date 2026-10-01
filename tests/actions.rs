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
#[test]
fn text_limits_count_utf8_and_reject_case_expansion_over_budget() {
    use nebula_paste::text_actions::{MAX_TEXT, Transform};
    let input = "ΐ".repeat(MAX_TEXT / 2);
    assert!(input.len() <= MAX_TEXT);
    assert!(Transform::Upper.apply(&input).is_err());
}
