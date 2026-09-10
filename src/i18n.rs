//! French and English UI; user clipboard content is never translated.
use std::cell::Cell;
thread_local! { static ENGLISH: Cell<bool> = const { Cell::new(false) }; }
pub fn set_language(language: &str) {
    let english = match language {
        "en" => true,
        "fr" => false,
        _ => !["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .find_map(|key| std::env::var(key).ok().filter(|v| !v.is_empty()))
            .unwrap_or_else(|| "en".into())
            .starts_with("fr"),
    };
    ENGLISH.with(|value| value.set(english));
}
pub fn english() -> bool {
    ENGLISH.with(Cell::get)
}
#[macro_export]
macro_rules! tr {
    ($fr:literal, $en:literal) => {
        if $crate::i18n::english() { $en } else { $fr }
    };
}
#[macro_export]
macro_rules! tr_format {
    ($fr:literal, $en:literal $(, $arg:expr)* $(,)?) => {
        if $crate::i18n::english() { format!($en $(, $arg)*) } else { format!($fr $(, $arg)*) }
    };
}
#[cfg(test)]
mod tests {
    #[test]
    fn languages_and_interpolated_messages() {
        super::set_language("en");
        assert_eq!(crate::tr!("Copier", "Copy"), "Copy");
        assert_eq!(crate::tr_format!("{} copies", "{} clips", 3), "3 clips");
        super::set_language("fr");
        assert_eq!(crate::tr!("Copier", "Copy"), "Copier");
    }
}
