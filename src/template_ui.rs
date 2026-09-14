//! Template screens use COSMIC widgets and keep field values only in memory.
use cosmic::{Element, iced::Length, widget};
use nebula_paste::{
    storage::Store,
    templates::{self, Archive, Conflict, Template},
    tr, tr_format,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
    Filter(String),
    New,
    Edit(String),
    Title(String),
    Collection(String),
    Body(widget::text_editor::Action),
    Save,
    Back,
    Discard,
    Use(String),
    Value(usize, String),
    Copy,
    AskDelete(String),
    CancelDelete,
    Delete,
    Import,
    Imported(Result<Option<Archive>, String>),
    Policy(Conflict),
    ApplyImport,
    Export,
    Exported(Result<bool, String>),
}
pub enum Effect {
    None,
    Copy(String),
    Import,
    Export(Archive),
}
struct Editor {
    id: Option<String>,
    title: String,
    collection: String,
    body: widget::text_editor::Content<cosmic::Renderer>,
    dirty: bool,
}
struct Filling {
    template: Template,
    values: Vec<(String, String)>,
}
pub struct State {
    items: Vec<Template>,
    pub search_id: cosmic::iced::widget::Id,
    query: String,
    filter: String,
    editing: Option<Editor>,
    filling: Option<Filling>,
    deleting: Option<String>,
    incoming: Option<Archive>,
    policy: Conflict,
    discard: bool,
    pub busy: bool,
    pub note: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            items: vec![],
            search_id: cosmic::iced::widget::Id::unique(),
            query: String::new(),
            filter: String::new(),
            editing: None,
            filling: None,
            deleting: None,
            incoming: None,
            policy: Conflict::Skip,
            discard: false,
            busy: false,
            note: String::new(),
        }
    }
}
impl State {
    pub fn at_root(&self) -> bool {
        self.editing.is_none()
            && self.filling.is_none()
            && self.incoming.is_none()
            && self.deleting.is_none()
    }
    pub fn refresh(&mut self, store: &Store) {
        match store.templates() {
            Ok(items) => self.items = items,
            Err(e) => self.note = e,
        }
    }
    pub fn clear_values(&mut self) {
        self.filling = None;
    }
    pub fn from_clip(&mut self, title: &str, body: &str, collection: &str) {
        if self.editing.is_some() {
            self.note = tr!(
                "Termine le modèle en cours avant d’en créer un autre.",
                "Finish the current template before creating another."
            )
            .into();
            return;
        }
        self.filling = None;
        self.editing = Some(Editor {
            id: None,
            title: title.chars().take(100).collect(),
            body: widget::text_editor::Content::with_text(body),
            collection: collection.into(),
            dirty: true,
        });
    }
    pub fn update(&mut self, msg: Message, store: &Store) -> Result<Effect, String> {
        match msg {
            Message::Search(s) => self.query = s,
            Message::Filter(s) => self.filter = s,
            Message::New => {
                self.editing = Some(Editor {
                    id: None,
                    title: String::new(),
                    collection: self.filter.clone(),
                    body: widget::text_editor::Content::new(),
                    dirty: false,
                })
            }
            Message::Edit(id) => {
                if let Some(t) = self.items.iter().find(|t| t.id == id) {
                    self.editing = Some(Editor {
                        id: Some(id),
                        title: t.title.clone(),
                        collection: t.collection.clone(),
                        body: widget::text_editor::Content::with_text(&t.body),
                        dirty: false,
                    });
                }
            }
            Message::Title(s) => {
                if let Some(e) = &mut self.editing {
                    e.title = s;
                    e.dirty = true;
                }
            }
            Message::Collection(s) => {
                if let Some(e) = &mut self.editing {
                    e.collection = s;
                    e.dirty = true;
                }
            }
            Message::Body(action) => {
                if let Some(e) = &mut self.editing {
                    e.dirty |= matches!(action, widget::text_editor::Action::Edit(_));
                    e.body.perform(action);
                }
            }
            Message::Save => {
                if let Some(e) = &self.editing {
                    store.save_template(
                        e.id.as_deref(),
                        &e.title,
                        &e.body.text(),
                        &e.collection,
                    )?;
                    self.editing = None;
                    self.discard = false;
                    self.refresh(store);
                    self.note = tr!("Modèle enregistré", "Template saved").into();
                }
            }
            Message::Back => {
                if self.editing.as_ref().is_some_and(|e| e.dirty) {
                    self.discard = true;
                } else {
                    self.editing = None;
                }
                self.filling = None;
                self.incoming = None;
                self.deleting = None;
            }
            Message::Discard => {
                self.editing = None;
                self.discard = false;
            }
            Message::Use(id) => {
                if let Some(t) = self.items.iter().find(|t| t.id == id) {
                    let values = templates::fields(&t.body)?
                        .into_iter()
                        .map(|s| (s, String::new()))
                        .collect();
                    self.filling = Some(Filling {
                        template: t.clone(),
                        values,
                    });
                }
            }
            Message::Value(i, s) => {
                if let Some(f) = &mut self.filling {
                    if let Some(v) = f.values.get_mut(i) {
                        v.1 = s;
                    }
                }
            }
            Message::Copy => {
                if let Some(f) = &self.filling {
                    return Ok(Effect::Copy(templates::expand(
                        &f.template.body,
                        &f.values.iter().cloned().collect(),
                    )?));
                }
            }
            Message::AskDelete(id) => self.deleting = Some(id),
            Message::CancelDelete => self.deleting = None,
            Message::Delete => {
                if let Some(id) = &self.deleting {
                    store.delete_template(id)?;
                    self.deleting = None;
                    self.refresh(store);
                    self.note = tr!("Modèle supprimé", "Template deleted").into();
                }
            }
            Message::Import => {
                if !self.busy {
                    self.busy = true;
                    return Ok(Effect::Import);
                }
            }
            Message::Imported(result) => {
                self.busy = false;
                self.incoming = result?;
                self.policy = Conflict::Skip;
            }
            Message::Policy(p) => self.policy = p,
            Message::ApplyImport => {
                if let Some(a) = &self.incoming {
                    let result = store.import_templates(a, self.policy)?;
                    self.incoming = None;
                    self.refresh(store);
                    self.note = tr_format!(
                        "Import : {} ajoutés, {} remplacés, {} ignorés.",
                        "Import: {} added, {} replaced, {} skipped.",
                        result.added,
                        result.replaced,
                        result.skipped
                    );
                }
            }
            Message::Export => {
                if !self.busy {
                    let a = Archive::new(store.templates()?);
                    a.validate()?;
                    self.busy = true;
                    return Ok(Effect::Export(a));
                }
            }
            Message::Exported(result) => {
                self.busy = false;
                if result? {
                    self.note = tr!("Modèles exportés", "Templates exported").into();
                }
            }
        }
        Ok(Effect::None)
    }
    pub fn view<'a>(&'a self, collections: &'a [String], compact: bool) -> Element<'a, Message> {
        let mut body = widget::column([])
            .spacing(10)
            .push(widget::text(tr!("Modèles", "Templates")).size(20))
            .push(
                widget::text(tr!(
                    "Tes textes réutilisables, conservés séparément de l’historique.",
                    "Reusable text, stored separately from clipboard history."
                ))
                .size(12),
            );
        if self.busy {
            body = body.push(
                widget::text(tr!(
                    "Choix du fichier / transfert en cours…",
                    "Choosing a file / transfer in progress…"
                ))
                .size(12),
            );
        }
        if !self.note.is_empty() {
            body = body.push(widget::text(&self.note).size(12));
        }
        if let Some(e) = &self.editing {
            body = body
                .push(widget::text_input(tr!("Titre", "Title"), &e.title).on_input(Message::Title))
                .push(
                    widget::text_input(
                        tr!("Collection (facultatif)", "Collection (optional)"),
                        &e.collection,
                    )
                    .on_input(Message::Collection),
                );
            let choices: Vec<Element<'a, Message>> = collections
                .iter()
                .map(|c| {
                    widget::button::text(c)
                        .on_press(Message::Collection(c.clone()))
                        .into()
                })
                .collect();
            if !choices.is_empty() {
                body =
                    body.push(widget::scrollable(widget::flex_row(choices).spacing(4)).height(65));
            }
            body=body.push(widget::text(tr!("Champs : {{nom}}, {{date}}, {{serveur}}… Doubles accolades réservées aux champs.","Fields: {{name}}, {{date}}, {{server}}… Double braces are reserved for fields.")).size(12))
                .push(widget::TextEditor::new(&e.body).height(if compact {150}else{230}).on_action(Message::Body));
            if let Err(err) = templates::fields(&e.body.text()) {
                body = body.push(widget::text(err).size(12));
            }
            body = body.push(
                widget::row([])
                    .spacing(8)
                    .push(
                        widget::button::suggested(tr!("Enregistrer", "Save"))
                            .on_press(Message::Save),
                    )
                    .push(widget::button::text(tr!("Retour", "Back")).on_press(Message::Back)),
            );
            if self.discard {
                body = body
                    .push(widget::text(tr!(
                        "Abandonner les modifications ?",
                        "Discard changes?"
                    )))
                    .push(
                        widget::button::destructive(tr!("Abandonner", "Discard"))
                            .on_press(Message::Discard),
                    );
            }
        } else if let Some(f) = &self.filling {
            body = body.push(widget::text(&f.template.title).size(17));
            let mut inputs = widget::column([]).spacing(8);
            for (i, (name, value)) in f.values.iter().enumerate() {
                inputs = inputs
                    .push(widget::text(name).size(12))
                    .push(widget::text_input(name, value).on_input(move |v| Message::Value(i, v)));
            }
            if !f.values.is_empty() {
                body =
                    body.push(widget::scrollable(inputs).height(if compact { 140 } else { 220 }));
            }
            let values: HashMap<_, _> = f.values.iter().cloned().collect();
            let preview = templates::expand(&f.template.body, &values);
            let valid = preview.is_ok();
            body=body.push(widget::text(tr!("Aperçu avant copie", "Preview before copying")).size(13))
                .push(widget::scrollable(widget::text(preview.unwrap_or_else(|_|f.template.body.clone())).size(13)).height(if compact {120}else{180}))
                .push(widget::text(tr!("Les valeurs restent en mémoire jusqu’à la fermeture de cette vue. Le texte copié peut rejoindre l’historique.","Values stay in memory until this view closes. Copied text may enter clipboard history.")).size(11))
                .push(widget::row([]).spacing(8)
                    .push(widget::button::suggested(tr!("Copier le texte", "Copy text")).on_press_maybe(valid.then_some(Message::Copy)))
                    .push(widget::button::text(tr!("Retour", "Back")).on_press(Message::Back)));
        } else if let Some(a) = &self.incoming {
            let conflicts = a
                .templates
                .iter()
                .filter(|t| self.items.iter().any(|old| old.id == t.id))
                .count();
            body = body.push(widget::text(tr_format!(
                "{} modèles · {} conflits d’identifiant",
                "{} templates · {} ID conflicts",
                a.templates.len(),
                conflicts
            )));
            let mut entries = widget::column([]).spacing(5);
            for t in &a.templates {
                entries = entries.push(
                    widget::text(format!(
                        "{} · {}{}",
                        t.title,
                        t.collection,
                        if self.items.iter().any(|old| old.id == t.id) {
                            tr!(" · conflit", " · conflict")
                        } else {
                            ""
                        }
                    ))
                    .size(12),
                );
            }
            body = body.push(widget::scrollable(entries).height(160));
            for (p, label) in [
                (
                    Conflict::Skip,
                    tr!("Ignorer les conflits", "Skip conflicts"),
                ),
                (Conflict::KeepBoth, tr!("Conserver les deux", "Keep both")),
                (
                    Conflict::Replace,
                    tr!("Remplacer les conflits", "Replace conflicts"),
                ),
            ] {
                body = body.push(
                    widget::button::text(label)
                        .class(crate::skin::button(self.policy == p, 8.0, false))
                        .on_press(Message::Policy(p)),
                );
            }
            body = body.push(
                widget::row([])
                    .spacing(8)
                    .push(
                        widget::button::suggested(tr!("Confirmer l’import", "Confirm import"))
                            .on_press(Message::ApplyImport),
                    )
                    .push(widget::button::text(tr!("Annuler", "Cancel")).on_press(Message::Back)),
            );
        } else {
            let controls: Vec<Element<'a, Message>> = vec![
                widget::button::suggested(tr!("Nouveau", "New"))
                    .on_press(Message::New)
                    .into(),
                widget::button::text(tr!("Importer…", "Import…"))
                    .on_press_maybe((!self.busy).then_some(Message::Import))
                    .into(),
                widget::button::text(tr!("Exporter tout…", "Export all…"))
                    .on_press_maybe(
                        (!self.busy && !self.items.is_empty()).then_some(Message::Export),
                    )
                    .into(),
            ];
            body = body.push(widget::flex_row(controls).spacing(6)).push(
                widget::search_input(
                    tr!("Rechercher un modèle…", "Search templates…"),
                    &self.query,
                )
                .id(self.search_id.clone())
                .on_input(Message::Search),
            );
            let mut filters: Vec<Element<'a, Message>> = vec![
                widget::button::text(tr!("Toutes les collections", "All collections"))
                    .class(crate::skin::button(self.filter.is_empty(), 8.0, false))
                    .on_press(Message::Filter(String::new()))
                    .into(),
            ];
            for c in collections {
                filters.push(
                    widget::button::text(c)
                        .class(crate::skin::button(self.filter == *c, 8.0, false))
                        .on_press(Message::Filter(c.clone()))
                        .into(),
                );
            }
            body = body.push(widget::scrollable(widget::flex_row(filters).spacing(4)).height(65));
            let mut list = widget::column([]).spacing(8);
            let mut count = 0;
            for t in self
                .items
                .iter()
                .filter(|t| t.matches(&self.query, &self.filter))
            {
                count += 1;
                let row = widget::column([])
                    .spacing(4)
                    .push(widget::button::text(&t.title).on_press(Message::Use(t.id.clone())))
                    .push(
                        widget::text(
                            t.body
                                .lines()
                                .next()
                                .unwrap_or("")
                                .chars()
                                .take(80)
                                .collect::<String>(),
                        )
                        .size(12),
                    )
                    .push(
                        widget::row([])
                            .spacing(6)
                            .push(
                                widget::button::text(tr!("Modifier", "Edit"))
                                    .on_press(Message::Edit(t.id.clone())),
                            )
                            .push(
                                widget::button::text(tr!("Supprimer…", "Delete…"))
                                    .on_press(Message::AskDelete(t.id.clone())),
                            ),
                    );
                list = list.push(
                    widget::container(row)
                        .padding(8)
                        .width(Length::Fill)
                        .class(cosmic::theme::Container::Card),
                );
            }
            if count == 0 {
                list = list.push(widget::text(tr!(
                    "Aucun modèle. Crée ton premier texte réutilisable.",
                    "No templates. Create your first reusable text."
                )));
            }
            body = body.push(widget::scrollable(list).height(if compact { 210 } else { 340 }));
            if let Some(id) = &self.deleting {
                let title = self
                    .items
                    .iter()
                    .find(|t| t.id == *id)
                    .map_or("", |t| t.title.as_str());
                body = body
                    .push(widget::text(tr_format!(
                        "Supprimer « {title} » ?",
                        "Delete “{title}”?"
                    )))
                    .push(
                        widget::row([])
                            .spacing(8)
                            .push(
                                widget::button::destructive(tr!("Supprimer", "Delete"))
                                    .on_press(Message::Delete),
                            )
                            .push(
                                widget::button::text(tr!("Annuler", "Cancel"))
                                    .on_press(Message::CancelDelete),
                            ),
                    );
            }
        }
        body.into()
    }
}

pub async fn import_file() -> Result<Option<Archive>, String> {
    use cosmic::dialog::file_chooser;
    let response = match file_chooser::open::Dialog::new()
        .title(tr!("Importer des modèles", "Import templates"))
        .filter(file_chooser::FileFilter::new("JSON").glob("*.json"))
        .open_file()
        .await
    {
        Ok(r) => r,
        Err(file_chooser::Error::Cancelled) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let path = response
        .url()
        .to_file_path()
        .map_err(|_| tr!("Fichier local requis", "Local file required").to_string())?;
    tokio::task::spawn_blocking(move || Archive::read(&path))
        .await
        .map_err(|e| e.to_string())?
        .map(Some)
}
pub async fn export_file(archive: Archive) -> Result<bool, String> {
    use cosmic::dialog::file_chooser;
    let response = match file_chooser::save::Dialog::new()
        .title(tr!(
            "Exporter les modèles vers un nouveau fichier",
            "Export templates to a new file"
        ))
        .file_name("nebula-paste-templates.json")
        .save_file()
        .await
    {
        Ok(r) => r,
        Err(file_chooser::Error::Cancelled) => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    let Some(url) = response.url() else {
        return Ok(false);
    };
    let path = url
        .to_file_path()
        .map_err(|_| tr!("Fichier local requis", "Local file required").to_string())?;
    if path.exists() {
        return Err(tr!(
            "Choisis un nouveau nom : le fichier existe déjà.",
            "Choose a new name: the file already exists."
        )
        .into());
    }
    tokio::task::spawn_blocking(move || archive.export_new(&path))
        .await
        .map_err(|e| e.to_string())??;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_editor_keeps_draft_then_save_and_fill_preserve_source() {
        let store = Store::in_memory().unwrap();
        let mut state = State::default();
        state.from_clip("Réponse", "Bonjour {{nom", "Travail");
        assert!(state.update(Message::Save, &store).is_err());
        assert!(state.editing.is_some());
        assert!(store.templates().unwrap().is_empty());
        state.editing.as_mut().unwrap().body =
            widget::text_editor::Content::with_text("Bonjour {{nom}} / {{nom}}");
        state.update(Message::Save, &store).unwrap();
        let t = store.templates().unwrap().remove(0);
        state.update(Message::Use(t.id.clone()), &store).unwrap();
        assert!(state.update(Message::Copy, &store).is_err());
        state
            .update(Message::Value(0, "Zoë {{literal}}".into()), &store)
            .unwrap();
        match state.update(Message::Copy, &store).unwrap() {
            Effect::Copy(text) => {
                assert!(text.starts_with("Bonjour Zoë {{literal}} / Zoë {{literal}}"))
            }
            _ => panic!("expected literal text"),
        }
        assert_eq!(store.templates().unwrap()[0], t);
        assert!(store.load().unwrap().is_empty());
        state.clear_values();
        assert!(state.filling.is_none());
    }
    #[test]
    fn back_requires_explicit_discard_and_import_waits_for_confirmation() {
        let store = Store::in_memory().unwrap();
        let mut state = State::default();
        state.from_clip("Draft", "Body", "");
        state.update(Message::Back, &store).unwrap();
        assert!(state.editing.is_some() && state.discard);
        state.update(Message::Discard, &store).unwrap();
        assert!(state.at_root());
        let source = Store::in_memory().unwrap();
        source.save_template(None, "Imported", "Text", "").unwrap();
        let archive = Archive::new(source.templates().unwrap());
        state
            .update(Message::Imported(Ok(Some(archive))), &store)
            .unwrap();
        assert!(store.templates().unwrap().is_empty());
        state.update(Message::ApplyImport, &store).unwrap();
        assert_eq!(store.templates().unwrap().len(), 1);
    }
}
