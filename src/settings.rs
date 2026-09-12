//! Préférences persistantes : fichier texte simple, valeurs invalides ignorées.
use crate::{tr, tr_format};
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

/// Durées de rétention proposées, en jours. `0` conserve sans limite d’âge.
pub const RETENTIONS: [u32; 5] = [0, 1, 7, 30, 365];
/// Langues du moteur OCR embarqué, dans l’ordre de bascule.
pub const OCR_LANGUAGES: [&str; 3] = ["fra+eng", "fra", "eng"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density {
    /// Grille de cartes avec aperçu : confort de lecture.
    Comfortable,
    /// Liste d’une colonne : petits écrans et fortes mises à l’échelle.
    Compact,
}

impl Density {
    pub fn label(self) -> &'static str {
        match self {
            Self::Comfortable => tr!("Grille", "Grid"),
            Self::Compact => tr!("Liste compacte", "Compact list"),
        }
    }
    pub fn next(self) -> Self {
        match self {
            Self::Comfortable => Self::Compact,
            Self::Compact => Self::Comfortable,
        }
    }
    fn key(self) -> &'static str {
        match self {
            Self::Comfortable => "comfortable",
            Self::Compact => "compact",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        match value {
            "comfortable" => Some(Self::Comfortable),
            "compact" => Some(Self::Compact),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub density: Density,
    /// Toujours l’une des valeurs de `OCR_LANGUAGES`.
    pub ocr_language: &'static str,
    /// Toujours l’une des valeurs de `RETENTIONS`.
    pub retention_days: u32,
    /// 0 : copie seule, 1 : Ctrl+V, 2 : Ctrl+Maj+V.
    pub paste_mode: u8,
    pub keep_open: bool,
    pub ocr_indexing: bool,
    pub ui_language: &'static str,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            density: Density::Comfortable,
            ocr_language: OCR_LANGUAGES[0],
            retention_days: 0,
            paste_mode: 0,
            keep_open: false,
            ocr_indexing: false,
            ui_language: "auto",
        }
    }
}

impl Settings {
    /// Emplacement par défaut ; `None` si le dossier de configuration est introuvable.
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_local_dir().map(|p| p.join("nebula-paste/settings.conf"))
    }
    /// Lit les préférences ; un fichier absent, illisible ou partiellement invalide
    /// rend les valeurs par défaut plutôt qu’une erreur : l’applet doit démarrer.
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .map(|text| Self::parse(&text))
            .unwrap_or_default()
    }
    pub fn parse(text: &str) -> Self {
        let mut settings = Self::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "density" => {
                    if let Some(density) = Density::parse(value) {
                        settings.density = density;
                    }
                }
                "ocr-language" => {
                    if let Some(language) = OCR_LANGUAGES.into_iter().find(|l| *l == value) {
                        settings.ocr_language = language;
                    }
                }
                "retention-days" => {
                    if let Ok(days) = value.parse::<u32>()
                        && RETENTIONS.contains(&days)
                    {
                        settings.retention_days = days;
                    }
                }
                "ui-language" => {
                    settings.ui_language = match value {
                        "fr" => "fr",
                        "en" => "en",
                        _ => "auto",
                    }
                }
                "ocr-indexing" => settings.ocr_indexing = value == "true",
                "keep-open" => settings.keep_open = value == "true",
                "paste-mode" => {
                    if let Ok(mode) = value.parse::<u8>()
                        && mode < 3
                    {
                        settings.paste_mode = mode;
                    }
                }
                _ => {}
            }
        }
        settings
    }
    pub fn render(&self) -> String {
        format!(
            "# Préférences de Nebula Paste. Les valeurs inconnues sont ignorées.\n\
             density = {}\n\
             ocr-language = {}\n\
             retention-days = {}\n\
             paste-mode = {}\n\
             keep-open = {}\n\
             ui-language = {}\n\
             ocr-indexing = {}\n",
            self.density.key(),
            self.ocr_language,
            self.retention_days,
            self.paste_mode,
            self.keep_open,
            self.ui_language,
            self.ocr_indexing
        )
    }
    /// Écrit dans un fichier temporaire puis renomme : une interruption ne laisse
    /// jamais des préférences tronquées à la place des précédentes.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let parent = path.parent().ok_or("Chemin de préférences invalide")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temporary
            .write_all(self.render().as_bytes())
            .map_err(|e| e.to_string())?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        temporary.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    }
    /// Fait défiler la durée de rétention parmi `RETENTIONS`.
    pub fn cycle_retention(&mut self) {
        let index = RETENTIONS
            .iter()
            .position(|days| *days == self.retention_days)
            .unwrap_or(0);
        self.retention_days = RETENTIONS[(index + 1) % RETENTIONS.len()];
    }
    pub fn cycle_ocr_language(&mut self) {
        let index = OCR_LANGUAGES
            .iter()
            .position(|language| *language == self.ocr_language)
            .unwrap_or(0);
        self.ocr_language = OCR_LANGUAGES[(index + 1) % OCR_LANGUAGES.len()];
    }
    pub fn retention_label(&self) -> String {
        match self.retention_days {
            0 => tr!("Sans limite d’âge", "No age limit").into(),
            1 => tr!("1 jour", "1 day").into(),
            days => tr_format!("{days} jours", "{days} days"),
        }
    }
    pub fn ocr_label(&self) -> &'static str {
        match self.ocr_language {
            "fra" => tr!("Français", "French"),
            "eng" => tr!("Anglais", "English"),
            _ => tr!("Français + anglais", "French + English"),
        }
    }
    pub fn paste_label(&self) -> &'static str {
        match self.paste_mode {
            1 => tr!("Coller · Ctrl+V", "Paste · Ctrl+V"),
            2 => tr!("Coller · Terminal", "Paste · Terminal"),
            _ => tr!("Copier seulement", "Copy only"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_values_fall_back_without_losing_valid_ones() {
        let settings = Settings::parse(
            "# commentaire\ndensity = compact\nocr-language = klingon\nretention-days = 12\npaste-mode = 9\nbruit\n",
        );
        assert_eq!(settings.density, Density::Compact);
        assert_eq!(settings.ocr_language, "fra+eng");
        assert_eq!(settings.retention_days, 0);
        assert_eq!(settings.paste_mode, 0);
    }

    #[test]
    fn preferences_survive_a_write_and_read_cycle() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config/settings.conf");
        let mut settings = Settings::default();
        settings.density = Density::Compact;
        settings.paste_mode = 2;
        settings.cycle_retention();
        settings.cycle_ocr_language();
        settings.save(&path).unwrap();
        assert_eq!(Settings::load(&path), settings);
        assert_eq!(settings.retention_days, 1);
        assert_eq!(settings.ocr_language, "fra");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn missing_file_and_cycles_stay_within_known_values() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            Settings::load(&dir.path().join("absent.conf")),
            Settings::default()
        );
        let mut settings = Settings::default();
        for _ in 0..RETENTIONS.len() {
            settings.cycle_retention();
            assert!(RETENTIONS.contains(&settings.retention_days));
        }
        assert_eq!(settings.retention_days, 0);
        for _ in 0..OCR_LANGUAGES.len() {
            settings.cycle_ocr_language();
            assert!(OCR_LANGUAGES.contains(&settings.ocr_language));
        }
        assert_eq!(settings.ocr_language, "fra+eng");
    }
}
