//! Bounded, literal text operations. Never executes source text.
use crate::tr;
use serde::{Deserialize, Serialize};
pub const MAX_TEXT: usize = 262_144;
pub const MAX_PARTS: usize = 100;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub title: String,
    pub text: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Transform {
    #[default]
    Original,
    Upper,
    Lower,
    Trim,
    RemoveBlank,
}
impl Transform {
    pub const ALL: [Self; 5] = [
        Self::Original,
        Self::Upper,
        Self::Lower,
        Self::Trim,
        Self::RemoveBlank,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Original => tr!("Original", "Original"),
            Self::Upper => tr!("MAJUSCULES", "UPPERCASE"),
            Self::Lower => tr!("minuscules", "lowercase"),
            Self::Trim => tr!("Nettoyer les espaces", "Trim whitespace"),
            Self::RemoveBlank => tr!("Retirer les lignes vides", "Remove blank lines"),
        }
    }
    pub fn apply(self, text: &str) -> Result<String, String> {
        check(text)?;
        let out = match self {
            Self::Original => text.into(),
            Self::Upper => text.to_uppercase(),
            Self::Lower => text.to_lowercase(),
            Self::Trim => text.lines().map(str::trim).collect::<Vec<_>>().join("\n"),
            Self::RemoveBlank => text
                .lines()
                .filter(|l| !l.trim().is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
        };
        check(&out)?;
        Ok(out)
    }
}
pub fn check(s: &str) -> Result<(), String> {
    if s.len() > MAX_TEXT || s.contains('\0') {
        Err(tr!(
            "Texte trop grand (256 Kio maximum) ou invalide.",
            "Text too large (256 KiB maximum) or invalid."
        )
        .into())
    } else {
        Ok(())
    }
}
pub fn assemble(parts: &[Part], separator: &str, transform: Transform) -> Result<String, String> {
    if parts.is_empty() || parts.len() > MAX_PARTS {
        return Err(tr!("Choisis de 1 à 100 textes.", "Choose 1 to 100 text items.").into());
    }
    check(separator)?;
    let mut out = String::new();
    for (i, p) in parts.iter().enumerate() {
        check(&p.text)?;
        let sep = if i == 0 { "" } else { separator };
        if out.len() + sep.len() + p.text.len() > MAX_TEXT {
            return Err(tr!(
                "Assemblage limité à 256 Kio.",
                "Assembly is limited to 256 KiB."
            )
            .into());
        }
        out.push_str(sep);
        out.push_str(&p.text);
    }
    transform.apply(&out)
}
#[derive(Default)]
pub struct Queue {
    pub parts: Vec<Part>,
    pub next: usize,
}
impl Queue {
    pub fn replace(&mut self, parts: &[Part], transform: Transform) -> Result<(), String> {
        assemble(parts, "", Transform::Original)?;
        let next = parts
            .iter()
            .map(|p| {
                Ok(Part {
                    title: p.title.clone(),
                    text: transform.apply(&p.text)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        self.parts = next;
        self.next = 0;
        Ok(())
    }
    pub fn current(&self) -> Option<&Part> {
        self.parts.get(self.next)
    }
    pub fn complete(&mut self, success: bool) {
        if success && self.next < self.parts.len() {
            self.next += 1;
        }
    }
    pub fn back(&mut self) {
        self.next = self.next.saturating_sub(1);
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Preferences {
    pub bindings: Vec<String>,
    pub slots: [Option<String>; 5],
    pub clicks: [u8; 3],
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            bindings: [
                "CTRL+F",
                "SPACE",
                "CTRL+SHIFT+C",
                "CTRL+SHIFT+A",
                "CTRL+SHIFT+N",
                "CTRL+SHIFT+B",
                "CTRL+SHIFT+F",
                "ALT+1",
                "ALT+2",
                "ALT+3",
                "ALT+4",
                "ALT+5",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            slots: Default::default(),
            clicks: [0; 3],
        }
    }
}
pub fn shortcut_label(i: usize) -> String {
    match i {
        0 => tr!("Rechercher", "Search").into(),
        1 => tr!("Aperçu", "Preview").into(),
        2 => tr!("Copier en texte brut", "Copy plain text").into(),
        3 => tr!("Actions", "Actions").into(),
        4 => tr!("Copier le suivant", "Copy next").into(),
        5 => tr!("Reculer dans la file", "Previous queue item").into(),
        6 => tr!("Favoris", "Favorites").into(),
        _ => format!("{} {}", tr!("Favori rapide", "Quick favorite"), i - 6),
    }
}
pub fn canonical(input: &str) -> Result<String, String> {
    let upper = input.trim().to_ascii_uppercase();
    if upper.is_empty() {
        return Ok(String::new());
    }
    let mut parts: Vec<_> = upper.split('+').map(str::trim).collect();
    let key = parts.pop().unwrap_or("");
    let valid_key = key == "SPACE"
        || key.len() == 1 && key.bytes().all(|c| c.is_ascii_alphanumeric())
        || key
            .strip_prefix('F')
            .and_then(|v| v.parse::<u8>().ok())
            .is_some_and(|v| (1..=12).contains(&v));
    let mut mods = Vec::new();
    for m in ["CTRL", "ALT", "SHIFT", "SUPER"] {
        if parts.contains(&m) {
            mods.push(m);
        }
    }
    if !valid_key
        || mods.len() != parts.len()
        || (mods.is_empty()
            && key != "SPACE"
            && !key
                .strip_prefix('F')
                .and_then(|v| v.parse::<u8>().ok())
                .is_some_and(|v| (1..=12).contains(&v)))
    {
        return Err(tr!(
            "Utilise Ctrl/Alt/Shift/Super + touche, Espace (SPACE) ou F1–F12.",
            "Use Ctrl/Alt/Shift/Super + key, SPACE or F1–F12."
        )
        .into());
    }
    mods.push(key);
    let value = mods.join("+");
    if [
        "CTRL+C",
        "CTRL+V",
        "CTRL+X",
        "CTRL+A",
        "CTRL+Z",
        "CTRL+Y",
        "CTRL+SHIFT+Z",
    ]
    .contains(&value.as_str())
        || value
            .strip_prefix("CTRL+")
            .is_some_and(|s| s.len() == 1 && s.as_bytes()[0].is_ascii_digit())
    {
        return Err(tr!(
            "Raccourci réservé à la saisie ou aux copies numérotées.",
            "Shortcut reserved for editing or numbered clips."
        )
        .into());
    }
    Ok(value)
}
impl Preferences {
    pub fn validate(&self) -> Result<(), String> {
        if self.bindings.len() != 12
            || self.clicks.iter().any(|c| *c > 2)
            || self
                .slots
                .iter()
                .flatten()
                .any(|id| id.len() > 128 || id.is_empty())
        {
            return Err("Invalid action preferences".into());
        }
        let mut used = std::collections::HashSet::new();
        for b in &self.bindings {
            if canonical(b)? != *b || !b.is_empty() && !used.insert(b) {
                return Err(tr!(
                    "Deux actions utilisent le même raccourci.",
                    "Two actions use the same shortcut."
                )
                .into());
            }
        }
        Ok(())
    }
    pub fn bind(&mut self, index: usize, value: &str) -> Result<(), String> {
        let mut new = self.clone();
        *new.bindings.get_mut(index).ok_or("Invalid action")? = canonical(value)?;
        new.validate()?;
        *self = new;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn parts() -> Vec<Part> {
        vec![
            Part {
                title: "A".into(),
                text: " été $(id) {{x}} \n\n".into(),
            },
            Part {
                title: "B".into(),
                text: "Straße".into(),
            },
        ]
    }
    #[test]
    fn assembly_is_literal_ordered_and_preserves_sources() {
        let p = parts();
        let before = p.clone();
        assert_eq!(
            assemble(&p, "|", Transform::Upper).unwrap(),
            " ÉTÉ $(ID) {{X}} \n\n|STRASSE"
        );
        assert_eq!(p, before);
        assert_eq!(
            Transform::RemoveBlank.apply(" a \n \n b ").unwrap(),
            " a \n b "
        );
        assert_eq!(Transform::Trim.apply(" a \n b ").unwrap(), "a\nb");
    }
    #[test]
    fn rejects_oversized_assembly_and_unicode_expansion() {
        assert!(assemble(&[], "", Transform::Original).is_err());
        let p = vec![
            Part {
                title: "".into(),
                text: "x".repeat(MAX_TEXT)
            };
            2
        ];
        assert!(assemble(&p, "", Transform::Original).is_err());
        assert!(Transform::Upper.apply(&"ﬃ".repeat(MAX_TEXT / 3)).is_ok());
        assert!(check("a\0b").is_err());
    }
    #[test]
    fn queue_advances_only_after_success_and_replace_is_atomic() {
        let mut q = Queue::default();
        q.replace(&parts(), Transform::Original).unwrap();
        q.complete(false);
        assert_eq!(q.next, 0);
        q.complete(true);
        assert_eq!(q.next, 1);
        q.back();
        assert_eq!(q.next, 0);
        assert!(q.replace(&[], Transform::Original).is_err());
        assert_eq!(q.parts.len(), 2);
        q.complete(true);
        q.complete(true);
        q.complete(true);
        assert!(q.current().is_none());
        assert_eq!(q.next, 2);
    }
    #[test]
    fn shortcuts_normalize_and_conflicts_do_not_mutate_preferences() {
        let mut p = Preferences::default();
        p.validate().unwrap();
        let original = p.clone();
        assert!(p.bind(3, "ctrl+f").is_err());
        assert_eq!(p, original);
        assert!(p.bind(3, "Ctrl+V").is_err());
        assert!(p.bind(3, "a").is_err());
        p.bind(3, " shift + ctrl + k ").unwrap();
        assert_eq!(p.bindings[3], "CTRL+SHIFT+K");
        p.bind(1, "").unwrap();
        assert!(p.bind(3, "CTRL+CTRL+K").is_err());
    }
}
