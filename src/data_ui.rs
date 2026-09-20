use cosmic::{Element, iced::Length, widget};
use nebula_paste::{
    backup::{self, Conflict, Validated},
    settings::Settings,
    tr, tr_format,
};
use std::{collections::HashSet, path::PathBuf, sync::Arc};
#[derive(Debug, Clone)]
pub enum Message {
    Backup,
    Restore,
    Diagnostic,
    Loaded(Result<Option<Arc<Validated>>, String>),
    Finished(Result<bool, String>),
    Policy(Conflict),
    Confirm,
    Cancel,
}
#[derive(Default)]
pub struct State {
    pub busy: bool,
    pub pending: Option<Arc<Validated>>,
    pub policy: Option<Conflict>,
    pub note: String,
}
impl State {
    pub fn view(&self, ids: &HashSet<String>) -> Element<'_, Message> {
        let enabled = !self.busy && self.pending.is_none();
        let mut c=widget::column([]).spacing(8)
   .push(widget::text(tr!("Sauvegarde et diagnostic","Backup and diagnostics")).size(16))
   .push(widget::text(tr!("Historique, favoris, collections et OCR. Les modèles ont leur propre export. Les fichiers pointés par les copies ne sont pas inclus.","History, favorites, collections and OCR. Templates have their own export. Files referenced by clips are not included.")).size(12))
   .push(widget::text(tr!("La sauvegarde contient vos copies en clair. Choisissez un emplacement privé.","Backups contain unencrypted clips. Choose a private location.")).size(12))
   .push(widget::button::standard(tr!("Sauvegarder l’historique…","Back up history…")).on_press_maybe(enabled.then_some(Message::Backup)))
   .push(widget::button::standard(tr!("Restaurer une sauvegarde…","Restore a backup…")).on_press_maybe(enabled.then_some(Message::Restore)))
   .push(widget::button::text(tr!("Exporter le diagnostic…","Export diagnostics…")).on_press_maybe(enabled.then_some(Message::Diagnostic)))
   .push(widget::text(tr!("Diagnostic : version, architecture, compteurs et réglages OCR/rétention. Aucun contenu, nom, chemin ou identifiant de copie.","Diagnostics: version, architecture, counts and OCR/retention settings. No contents, names, paths or clip identifiers.")).size(11));
        if let Some(data) = &self.pending {
            let s = data.summary(ids);
            c=c.push(widget::text(tr_format!("{} ajouts · {} conflits · {} favoris · {} collections · {:.1} Mio","{} additions · {} conflicts · {} favorites · {} collections · {:.1} MiB",s.added,s.conflicts,s.pinned,data.collections(),s.bytes as f64/1048576.0)))
    .push(widget::text(tr!("Un conflit désigne le même contenu. Choisir la sauvegarde remplace sa date, son classement et son statut favori. Les autres copies restent présentes. Si la capacité est dépassée, tout est annulé.","A conflict means identical content. Using the backup replaces its date, collection and favorite status. Other clips remain. If capacity is exceeded, everything is rolled back.")).size(12));
            for (p, label) in [
                (
                    Conflict::KeepLocal,
                    tr!("Conserver les données locales", "Keep local metadata"),
                ),
                (
                    Conflict::UseBackup,
                    tr!("Utiliser les données sauvegardées", "Use backup metadata"),
                ),
            ] {
                c = c.push(
                    widget::button::text(format!(
                        "{} {label}",
                        if self.policy == Some(p) { "●" } else { "○" }
                    ))
                    .on_press(Message::Policy(p)),
                );
            }
            c=c.push(widget::text(tr!("La restauration désactive la limite d’âge pour conserver les anciennes copies. Réappliquez ensuite la rétention souhaitée. L’index OCR importé est ignoré si la recherche d’images est désactivée.","Restore disables the age limit to retain old clips. Reapply your preferred retention afterwards. Imported OCR is omitted when image search is disabled.")).size(12))
    .push(widget::button::suggested(tr!("Restaurer et désactiver la limite d’âge","Restore and disable age limit")).on_press_maybe(self.policy.map(|_|Message::Confirm)))
    .push(widget::button::text(tr!("Annuler","Cancel")).on_press(Message::Cancel));
        }
        if self.busy {
            c = c.push(widget::text(tr!("Opération en cours…", "Working…")));
        }
        if !self.note.is_empty() {
            c = c.push(widget::text(&self.note).size(12));
        }
        widget::container(c).width(Length::Fill).padding(8).into()
    }
}
pub async fn choose_restore() -> Result<Option<Arc<Validated>>, String> {
    use cosmic::dialog::file_chooser;
    let r = match file_chooser::open::Dialog::new()
        .title(tr!("Restaurer l’historique", "Restore history"))
        .filter(file_chooser::FileFilter::new("JSON").glob("*.json"))
        .open_file()
        .await
    {
        Ok(r) => r,
        Err(file_chooser::Error::Cancelled) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let path = r.url().to_file_path().map_err(|_| "Local file required")?;
    tokio::task::spawn_blocking(move || backup::Archive::read(&path).map(|a| Some(Arc::new(a))))
        .await
        .map_err(|e| e.to_string())?
}
pub async fn export_snapshot(path: PathBuf, diagnostic: Option<Settings>) -> Result<bool, String> {
    use cosmic::dialog::file_chooser;
    let name = if diagnostic.is_some() {
        "nebula-paste-diagnostic.json"
    } else {
        "nebula-paste-history.json"
    };
    let r = match file_chooser::save::Dialog::new()
        .title(tr!("Exporter vers un nouveau fichier", "Export to a new file").into())
        .file_name(name.into())
        .save_file()
        .await
    {
        Ok(r) => r,
        Err(file_chooser::Error::Cancelled) => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    let Some(url) = r.url() else { return Ok(false) };
    let destination = url.to_file_path().map_err(|_| "Local file required")?;
    tokio::task::spawn_blocking(move || {
        let bytes = nebula_paste::storage::Store::export_snapshot(&path, diagnostic.as_ref())?;
        backup::write_new(&destination, &bytes)
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(true)
}
