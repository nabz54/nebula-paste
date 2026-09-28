//! Shared collection browser. Editing remains in the dedicated note/template editors.
use cosmic::{Element, iced::Length, widget};
use nebula_paste::{
    model::{self, Clip},
    storage::Store,
    tr, tr_format,
    workspace::{Key, Undo},
};
use std::collections::{HashMap, HashSet};
#[derive(Clone)]
struct Entry {
    kind: u8,
    id: String,
    title: String,
    body: String,
    source: String,
    collection: String,
    pinned: bool,
    can_convert: bool,
}
impl Entry {
    fn key(&self) -> Option<Key> {
        match self.kind {
            0 => Some(Key::Clip(self.id.clone())),
            1 => Some(Key::Note(self.id.clone())),
            _ => None,
        }
    }
    fn label(&self) -> &'static str {
        match self.kind {
            0 => tr!("Copie", "Clip"),
            1 => tr!("Note", "Note"),
            _ => tr!("Modèle", "Template"),
        }
    }
}
#[derive(Debug, Clone)]
pub enum Message {
    Actions,
    Search(String),
    Scope(u8),
    Collection(String),
    Layout,
    Order(String, bool),
    Select(Key),
    SelectAll,
    ClearSelection,
    Destination(String),
    Move,
    AskDelete,
    CancelDelete,
    Delete,
    Undo,
    Page(bool),
    NewNote,
    Notes,
    Open(u8, String),
    Convert(String),
    Manage,
}
pub enum Effect {
    Actions(Vec<nebula_paste::text_actions::Part>),
    None,
    NewNote,
    Notes,
    Open(u8, String),
    Convert(String),
    Manage,
}
pub struct State {
    entries: Vec<Entry>,
    collections: Vec<String>,
    pub search_id: cosmic::iced::widget::Id,
    query: String,
    scope: u8,
    collection: String,
    cards: bool,
    selected: HashSet<Key>,
    destination: String,
    confirm: bool,
    undo: Option<Undo>,
    page: usize,
    pub notice: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            entries: vec![],
            collections: vec![],
            search_id: cosmic::iced::widget::Id::unique(),
            query: String::new(),
            scope: 0,
            collection: String::new(),
            cards: true,
            selected: HashSet::new(),
            destination: String::new(),
            confirm: false,
            undo: None,
            page: 0,
            notice: String::new(),
        }
    }
}
impl State {
    pub fn refresh(
        &mut self,
        store: &Store,
        clips: &[Clip],
        ocr: &HashMap<String, String>,
    ) -> Result<(), String> {
        let notes = store.notes()?;
        let templates = store.templates()?;
        self.collections = store.collections()?;
        self.entries = clips
            .iter()
            .map(|c| Entry {
                kind: 0,
                id: c.id.clone(),
                title: c.title.clone(),
                source: c.text.clone(),
                body: format!(
                    "{}\n{}",
                    c.text,
                    ocr.get(&c.id).map(String::as_str).unwrap_or("")
                ),
                collection: c.category.clone(),
                pinned: c.pinned,
                can_convert: !matches!(c.kind, model::Kind::Image | model::Kind::Files),
            })
            .collect();
        self.entries.extend(notes.into_iter().map(|n| Entry {
            kind: 1,
            id: n.id,
            title: n.title,
            source: n.body.clone(),
            body: n.body,
            collection: n.collection,
            pinned: false,
            can_convert: false,
        }));
        self.entries.extend(templates.into_iter().map(|n| Entry {
            kind: 2,
            id: n.id,
            title: n.title,
            source: n.body.clone(),
            body: n.body,
            collection: n.collection,
            pinned: false,
            can_convert: false,
        }));
        if !self.collection.is_empty() && !self.collections.contains(&self.collection) {
            self.collection.clear();
        }
        if !self.destination.is_empty() && !self.collections.contains(&self.destination) {
            self.destination.clear();
        }
        self.cards = store.collection_cards(&self.collection);
        self.selected
            .retain(|key| self.entries.iter().any(|e| e.key().as_ref() == Some(key)));
        self.page = self.page.min(self.filtered().len().saturating_sub(1) / 40);
        Ok(())
    }
    fn filtered(&self) -> Vec<&Entry> {
        let query = model::search_fold(&self.query);
        self.entries
            .iter()
            .filter(|e| {
                (self.collection.is_empty() || e.collection == self.collection)
                    && match self.scope {
                        1 => e.kind == 0,
                        2 => e.kind == 1,
                        3 => e.kind == 2,
                        4 => e.pinned,
                        _ => true,
                    }
                    && {
                        let hay =
                            model::search_fold(&format!("{} {} {}", e.title, e.body, e.collection));
                        query.split_whitespace().all(|word| hay.contains(word))
                    }
            })
            .collect()
    }
    fn reset(&mut self) {
        self.page = 0;
        self.selected.clear();
        self.confirm = false;
    }
    pub fn update(&mut self, message: Message, store: &Store) -> Result<Effect, String> {
        match message {
            Message::Actions => {
                let entries: Vec<_> = self
                    .filtered()
                    .into_iter()
                    .filter(|e| e.key().is_some_and(|k| self.selected.contains(&k)))
                    .collect();
                if entries.iter().any(|e| e.kind == 0 && !e.can_convert) {
                    return Err(tr!(
                        "Sélectionne uniquement des copies textuelles et des notes.",
                        "Select only text clips and notes."
                    )
                    .into());
                }
                return Ok(Effect::Actions(
                    entries
                        .into_iter()
                        .map(|e| nebula_paste::text_actions::Part {
                            title: e.title.clone(),
                            text: e.source.clone(),
                        })
                        .collect(),
                ));
            }
            Message::Search(s) => {
                self.query = s;
                self.reset();
            }
            Message::Scope(s) => {
                self.scope = s;
                self.collection.clear();
                self.cards = store.collection_cards("");
                self.reset();
            }
            Message::Collection(s) => {
                self.collection = s;
                self.scope = 0;
                self.cards = store.collection_cards(&self.collection);
                self.reset();
            }
            Message::Layout => {
                let cards = !self.cards;
                store.set_collection_cards(&self.collection, cards)?;
                self.cards = cards;
            }
            Message::Order(name, up) => {
                store.move_collection(&name, up)?;
            }
            Message::Select(k) => {
                if !self.selected.remove(&k) {
                    self.selected.insert(k);
                }
                self.confirm = false;
            }
            Message::SelectAll => {
                self.selected = self.filtered().iter().filter_map(|e| e.key()).collect();
                self.confirm = false;
            }
            Message::ClearSelection => {
                self.selected.clear();
                self.confirm = false;
            }
            Message::Destination(s) => {
                self.destination = s;
                self.confirm = false;
            }
            Message::Move => {
                self.undo = Some(store.move_items(&self.selected, &self.destination)?);
                self.selected.clear();
                self.notice=tr!("Classement terminé. Annulation disponible jusqu’à la prochaine action groupée ou fermeture de l’application.","Filed. Undo is available until the next bulk action or application exit.").into();
            }
            Message::AskDelete => self.confirm = !self.selected.is_empty(),
            Message::CancelDelete => self.confirm = false,
            Message::Delete => {
                if self.confirm {
                    self.undo = Some(store.delete_items(&self.selected)?);
                    self.confirm = false;
                    self.selected.clear();
                    self.notice=tr!("Suppression effectuée. Annulation disponible jusqu’à la prochaine action groupée ou fermeture de l’application.","Deleted. Undo is available until the next bulk action or application exit.").into();
                }
            }
            Message::Undo => {
                if let Some(undo) = &self.undo {
                    store.undo_items(undo)?;
                    self.undo = None;
                    self.notice = tr!("Action annulée", "Action undone").into();
                }
            }
            Message::Page(next) => {
                let max = self.filtered().len().saturating_sub(1) / 40;
                self.page = if next {
                    (self.page + 1).min(max)
                } else {
                    self.page.saturating_sub(1)
                };
            }
            Message::NewNote => return Ok(Effect::NewNote),
            Message::Notes => return Ok(Effect::Notes),
            Message::Open(kind, id) => return Ok(Effect::Open(kind, id)),
            Message::Convert(id) => return Ok(Effect::Convert(id)),
            Message::Manage => return Ok(Effect::Manage),
        }
        Ok(Effect::None)
    }
    pub fn view<'a>(
        &'a self,
        width: f32,
        images: &'a HashMap<String, cosmic::iced::widget::image::Handle>,
    ) -> Element<'a, Message> {
        let mut side = widget::column([]).spacing(4);
        for (i, label) in [
            tr!("Tout", "All"),
            tr!("Historique", "History"),
            tr!("Notes", "Notes"),
            tr!("Modèles", "Templates"),
            tr!("Favoris", "Favorites"),
        ]
        .into_iter()
        .enumerate()
        {
            side = side.push(
                widget::button::text(label)
                    .class(crate::skin::button(
                        self.scope == i as u8 && self.collection.is_empty(),
                        8.0,
                        false,
                    ))
                    .on_press(Message::Scope(i as u8)),
            );
        }
        side = side.push(widget::text(tr!("Collections", "Collections")).size(13));
        for (i, c) in self.collections.iter().enumerate() {
            side = side.push(
                widget::row([])
                    .push(
                        widget::button::text(c)
                            .width(Length::Fill)
                            .class(crate::skin::button(self.collection == *c, 8.0, false))
                            .on_press(Message::Collection(c.clone())),
                    )
                    .push(
                        widget::button::text("↑")
                            .on_press_maybe((i > 0).then(|| Message::Order(c.clone(), true))),
                    )
                    .push(widget::button::text("↓").on_press_maybe(
                        (i + 1 < self.collections.len()).then(|| Message::Order(c.clone(), false)),
                    )),
            );
        }
        side = side.push(widget::button::text(tr!("Gérer…", "Manage…")).on_press(Message::Manage));
        let header = widget::flex_row(vec![
            widget::button::suggested(tr!("Nouvelle note", "New note"))
                .on_press(Message::NewNote)
                .into(),
            widget::button::text(tr!(
                "Importer / exporter les notes",
                "Import / export notes"
            ))
            .on_press(Message::Notes)
            .into(),
            widget::button::text(if self.cards {
                tr!("Vue : cartes", "View: cards")
            } else {
                tr!("Vue : liste", "View: list")
            })
            .on_press(Message::Layout)
            .into(),
        ])
        .spacing(6);
        let mut body = widget::column([]).spacing(8).push(header).push(
            widget::search_input(
                tr!(
                    "Rechercher copies, notes et modèles…",
                    "Search clips, notes and templates…"
                ),
                &self.query,
            )
            .id(self.search_id.clone())
            .on_input(Message::Search),
        );
        let filtered = self.filtered();
        body = body.push(
            widget::flex_row(vec![
                widget::text(tr_format!("{} résultats", "{} results", filtered.len())).into(),
                widget::button::text(tr!(
                    "Sélectionner toutes les copies et notes",
                    "Select all clips and notes"
                ))
                .on_press(Message::SelectAll)
                .into(),
            ])
            .spacing(8),
        );
        if !self.selected.is_empty() {
            body = body
                .push(widget::text(tr_format!(
                    "{} sélectionnés",
                    "{} selected",
                    self.selected.len()
                )))
                .push(
                    widget::flex_row(vec![
                        widget::button::text(tr!("Actions…", "Actions…"))
                            .on_press(Message::Actions)
                            .into(),
                        widget::button::text(tr!("Annuler la sélection", "Clear selection"))
                            .on_press(Message::ClearSelection)
                            .into(),
                        widget::button::text(tr!("Supprimer…", "Delete…"))
                            .on_press(Message::AskDelete)
                            .into(),
                    ])
                    .spacing(6),
                );
            let mut destinations = vec![
                widget::button::text(tr!("Sans collection", "Unfiled"))
                    .class(crate::skin::button(self.destination.is_empty(), 8.0, false))
                    .on_press(Message::Destination(String::new()))
                    .into(),
            ];
            for c in &self.collections {
                destinations.push(
                    widget::button::text(c)
                        .class(crate::skin::button(self.destination == *c, 8.0, false))
                        .on_press(Message::Destination(c.clone()))
                        .into(),
                );
            }
            body = body
                .push(widget::text(tr!(
                    "Collection de destination",
                    "Destination collection"
                )))
                .push(widget::flex_row(destinations).spacing(4))
                .push(
                    widget::button::text(tr!("Déplacer la sélection", "Move selection"))
                        .on_press(Message::Move),
                );
        }
        if self.confirm {
            body = body
                .push(widget::text(tr_format!(
                    "Supprimer ces {} copies et notes, y compris les favoris sélectionnés ?",
                    "Delete these {} clips and notes, including selected favorites?",
                    self.selected.len()
                )))
                .push(
                    widget::row([])
                        .push(
                            widget::button::text(tr!("Confirmer", "Confirm"))
                                .on_press(Message::Delete),
                        )
                        .push(
                            widget::button::text(tr!("Annuler", "Cancel"))
                                .on_press(Message::CancelDelete),
                        ),
                );
        }
        if self.undo.is_some() {
            body = body.push(
                widget::button::text(tr!(
                    "Annuler la dernière action groupée",
                    "Undo last bulk action"
                ))
                .on_press(Message::Undo),
            );
        }
        if !self.notice.is_empty() {
            body = body.push(widget::text(&self.notice).size(12));
        }
        let mut cards = Vec::new();
        let available = if width >= 600.0 { width - 236.0 } else { width };
        let columns = ((available + 10.0) / 240.0).floor().max(1.0);
        let card_width = ((available - (columns - 1.0) * 10.0) / columns - 2.0).max(200.0);
        for e in filtered.iter().skip(self.page * 40).take(40) {
            let label = if e.collection.is_empty() {
                e.label().to_owned()
            } else {
                format!("{} · {}", e.label(), e.collection)
            };
            let title = e.title.chars().take(80).collect::<String>();
            let mut actions: Vec<Element<'a, Message>> = vec![
                widget::button::text(match e.kind {
                    0 => tr!("Copier", "Copy"),
                    1 => tr!("Modifier", "Edit"),
                    _ => tr!("Utiliser", "Use"),
                })
                .on_press(Message::Open(e.kind, e.id.clone()))
                .into(),
            ];
            if let Some(key) = e.key() {
                actions.push(
                    widget::button::text(if self.selected.contains(&key) {
                        tr!("Sélectionné", "Selected")
                    } else {
                        tr!("Sélectionner", "Select")
                    })
                    .on_press(Message::Select(key))
                    .into(),
                );
            }
            if e.can_convert {
                actions.push(
                    widget::button::text("+ Note")
                        .on_press(Message::Convert(e.id.clone()))
                        .into(),
                );
            }
            let actions = widget::flex_row(actions).spacing(4);
            let content: Element<'a, Message> = if self.cards {
                let preview: Element<'a, Message> =
                    if let Some(image) = images.get(&e.id).filter(|_| e.kind == 0) {
                        widget::image(image.clone())
                            .height(96)
                            .width(Length::Fill)
                            .content_fit(cosmic::iced::ContentFit::Contain)
                            .into()
                    } else {
                        widget::text(e.body.chars().take(150).collect::<String>())
                            .size(12)
                            .into()
                    };
                widget::column([])
                    .spacing(6)
                    .push(widget::text(label).size(11))
                    .push(widget::container(widget::text(title).size(15)).height(42))
                    .push(widget::container(preview).height(96))
                    .push(actions)
                    .into()
            } else {
                let info = widget::column([])
                    .spacing(4)
                    .push(widget::text(label).size(11))
                    .push(widget::text(title).size(14));
                if width >= 720.0 {
                    widget::row([])
                        .spacing(8)
                        .align_y(cosmic::iced::Alignment::Center)
                        .push(widget::container(info).width(Length::Fill))
                        .push(widget::container(actions).width(270))
                        .into()
                } else {
                    widget::column([])
                        .spacing(4)
                        .push(info)
                        .push(actions)
                        .into()
                }
            };
            cards.push(
                widget::container(content)
                    .padding(12)
                    .width(if self.cards {
                        Length::Fixed(card_width)
                    } else {
                        Length::Fill
                    })
                    .class(cosmic::theme::Container::Card)
                    .into(),
            );
        }
        if self.cards {
            body = body.push(widget::flex_row(cards).spacing(10));
        } else {
            body = body.push(widget::column(cards).spacing(6));
        }
        if filtered.is_empty() {
            body = body.push(widget::text(tr!(
                "Aucun élément dans cette vue.",
                "No items in this view."
            )));
        }
        body = body.push(
            widget::row([])
                .spacing(8)
                .push(
                    widget::button::text("←")
                        .on_press_maybe((self.page > 0).then_some(Message::Page(false))),
                )
                .push(widget::text(format!(
                    "{} / {}",
                    self.page + 1,
                    filtered.len().max(1).div_ceil(40)
                )))
                .push(widget::button::text("→").on_press_maybe(
                    ((self.page + 1) * 40 < filtered.len()).then_some(Message::Page(true)),
                )),
        );
        if width < 600.0 {
            widget::column([]).push(side).push(body).spacing(12).into()
        } else {
            widget::row([])
                .spacing(16)
                .push(widget::container(side).width(220))
                .push(widget::container(body).width(Length::Fill))
                .into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unified_search_and_bulk_selection_keep_templates_separate() {
        let mut store = Store::in_memory().unwrap();
        let c = Clip::new("text/plain".into(), "Élodie copie".as_bytes().to_vec(), 1).unwrap();
        store.insert(&c).unwrap();
        store
            .save_note(None, "Élodie note", "persistent", "")
            .unwrap();
        store
            .save_template(None, "Élodie modèle", "{{nom}}", "")
            .unwrap();
        let mut state = State::default();
        state
            .refresh(&store, &store.load().unwrap(), &HashMap::new())
            .unwrap();
        state
            .update(Message::Search("elodie".into()), &store)
            .unwrap();
        assert_eq!(state.filtered().len(), 3);
        state.update(Message::SelectAll, &store).unwrap();
        assert_eq!(state.selected.len(), 2);
        state.update(Message::Delete, &store).unwrap();
        assert_eq!(store.notes().unwrap().len(), 1);
        state.update(Message::AskDelete, &store).unwrap();
        state.update(Message::Delete, &store).unwrap();
        assert!(store.notes().unwrap().is_empty());
        assert_eq!(store.templates().unwrap().len(), 1);
        state.update(Message::Undo, &store).unwrap();
        assert_eq!(store.notes().unwrap().len(), 1);
    }
}
