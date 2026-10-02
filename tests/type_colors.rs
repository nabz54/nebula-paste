use nebula_paste::settings::{Settings, TypeColors};
#[test]
fn modes_persist_and_unknown_values_keep_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.conf");
    for mode in [TypeColors::Subtle, TypeColors::Vivid, TypeColors::Off] {
        let mut s = Settings::default();
        s.type_colors = mode;
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path).type_colors, mode);
    }
    assert_eq!(
        Settings::parse("type-colors = unknown").type_colors,
        TypeColors::Subtle
    );
}
