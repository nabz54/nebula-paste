//! Text workbench and action preferences; source clips are never edited.
use cosmic::{Element, iced::Length, widget};
use nebula_paste::{
    model::{Clip, Kind},
    text_actions::{self as text, Part, Preferences, Queue, Transform},
    tr, tr_format,
};
#[derive(Debug, Clone)]
pub enum Message {
    Mode(Transform),
    Separator(u8),
    Custom(String),
    Move(usize, bool),
    Remove(usize),
    Copy,
    SaveNote,
    MakeQueue,
    Next,
    Previous,
    ClearQueue,
    Binding(usize, String),
    SaveBinding(usize),
    ResetBindings,
    Slot(usize),
    ClearSlot(usize),
    Favorite(usize),
    Click(usize),
}
pub enum Effect {
    None,
    Copy(String, bool),
    Note(String),
    Favorite(String),
    SavePreferences,
}
pub struct State {
    pub parts: Vec<Part>,
    pub queue: Queue,
    mode: Transform,
    separator: u8,
    custom: String,
    pub notice: String,
    pub busy: bool,
    pub drafts: Vec<String>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            parts: vec![],
            queue: Queue::default(),
            mode: Transform::Original,
            separator: 1,
            custom: String::new(),
            notice: String::new(),
            busy: false,
            drafts: Preferences::default().bindings,
        }
    }
}
impl State {
    pub fn load(&mut self, parts: Vec<Part>) -> Result<(), String> {
        if self.busy {
            return Err(tr!("Copie en cours", "Copy in progress").into());
        }
        text::assemble(&parts, "", Transform::Original)?;
        self.parts = parts;
        self.mode = Transform::Original;
        self.notice.clear();
        Ok(())
    }
    fn separator(&self) -> &str {
        match self.separator {
            0 => "",
            1 => "\n",
            2 => "\n\n",
            3 => ", ",
            _ => &self.custom,
        }
    }
    pub fn result(&self) -> Result<String, String> {
        text::assemble(&self.parts, self.separator(), self.mode)
    }
    pub fn update(
        &mut self,
        msg: Message,
        prefs: &mut Preferences,
        clips: &[Clip],
    ) -> Result<Effect, String> {
        if self.busy {
            return Err(tr!("Attends la fin de la copie.", "Wait for copying to finish.").into());
        }
        match msg {
            Message::Mode(m) => self.mode = m,
            Message::Separator(n) => self.separator = n.min(4),
            Message::Custom(s) => {
                if s.len() <= 1024 {
                    self.custom = s;
                }
            }
            Message::Move(i, up) => {
                if i < self.parts.len() {
                    let j = if up {
                        i.saturating_sub(1)
                    } else {
                        (i + 1).min(self.parts.len() - 1)
                    };
                    self.parts.swap(i, j);
                }
            }
            Message::Remove(i) => {
                if i < self.parts.len() {
                    self.parts.remove(i);
                }
            }
            Message::Copy => return Ok(Effect::Copy(self.result()?, false)),
            Message::SaveNote => return Ok(Effect::Note(self.result()?)),
            Message::MakeQueue => {
                self.queue.replace(&self.parts, self.mode)?;
                self.notice=tr!("File préparée en mémoire. Chaque copie nécessite ensuite Ctrl+V dans l’application cible.","Queue prepared in memory. After each copy, press Ctrl+V in the target application.").into();
            }
            Message::Next => {
                let item = self
                    .queue
                    .current()
                    .ok_or(tr!("File terminée ou vide.", "Queue is finished or empty."))?;
                return Ok(Effect::Copy(item.text.clone(), true));
            }
            Message::Previous => self.queue.back(),
            Message::ClearQueue => self.queue = Queue::default(),
            Message::Binding(i, s) => {
                if s.len() <= 80 {
                    if let Some(d) = self.drafts.get_mut(i) {
                        *d = s;
                    }
                }
            }
            Message::SaveBinding(i) => {
                prefs.bind(i, self.drafts.get(i).ok_or("Invalid shortcut")?)?;
                self.drafts = prefs.bindings.clone();
                return Ok(Effect::SavePreferences);
            }
            Message::ResetBindings => {
                prefs.bindings = Preferences::default().bindings;
                self.drafts = prefs.bindings.clone();
                return Ok(Effect::SavePreferences);
            }
            Message::Slot(i) => {
                if let Some(slot) = prefs.slots.get_mut(i) {
                    let favorites: Vec<_> = clips.iter().filter(|c| c.pinned).collect();
                    if favorites.is_empty() {
                        return Err(tr!(
                            "Ajoute d’abord une copie aux favoris.",
                            "First mark a clip as a favorite."
                        )
                        .into());
                    }
                    let next = slot
                        .as_ref()
                        .and_then(|id| favorites.iter().position(|c| &c.id == id))
                        .map_or(0, |n| (n + 1) % favorites.len());
                    *slot = Some(favorites[next].id.clone());
                    return Ok(Effect::SavePreferences);
                }
            }
            Message::ClearSlot(i) => {
                if let Some(s) = prefs.slots.get_mut(i) {
                    *s = None;
                    return Ok(Effect::SavePreferences);
                }
            }
            Message::Favorite(i) => {
                let id = prefs
                    .slots
                    .get(i)
                    .and_then(|s| s.as_ref())
                    .ok_or(tr!("Emplacement vide.", "Empty slot."))?;
                if !clips.iter().any(|c| &c.id == id && c.pinned) {
                    return Err(tr!(
                        "Ce favori n’est plus disponible. Réaffecte l’emplacement.",
                        "This favorite is unavailable. Reassign the slot."
                    )
                    .into());
                }
                return Ok(Effect::Favorite(id.clone()));
            }
            Message::Click(i) => {
                if let Some(c) = prefs.clicks.get_mut(i) {
                    *c = (*c + 1) % 3;
                    return Ok(Effect::SavePreferences);
                }
            }
        }
        Ok(Effect::None)
    }
    pub fn view<'a>(&'a self, prefs: &'a Preferences, clips: &'a [Clip]) -> Element<'a, Message> {
        let mut body = widget::column([])
            .spacing(10)
            .push(widget::text(tr!("Actions et clavier", "Actions and keyboard")).size(20));
        body=body.push(widget::text(tr!("Ouvre une copie texte depuis son aperçu, ou sélectionne des copies et notes dans la bibliothèque puis choisis « Actions ».","Open a text clip from its preview, or select clips and notes in the library and choose Actions.")).size(12));
        if !self.notice.is_empty() {
            body = body.push(widget::text(&self.notice).size(13));
        }
        if !self.parts.is_empty() {
            body = body.push(
                widget::text(tr!(
                    "Textes à assembler — ordre de sortie",
                    "Text assembly — output order"
                ))
                .size(16),
            );
            let mut list = widget::column([]).spacing(3);
            for (i, p) in self.parts.iter().enumerate() {
                list = list.push(
                    widget::row([])
                        .spacing(4)
                        .push(
                            widget::text(format!(
                                "{}. {}",
                                i + 1,
                                p.title.chars().take(40).collect::<String>()
                            ))
                            .width(Length::Fill),
                        )
                        .push(
                            widget::button::text("↑")
                                .on_press_maybe((i > 0).then_some(Message::Move(i, true))),
                        )
                        .push(widget::button::text("↓").on_press_maybe(
                            (i + 1 < self.parts.len()).then_some(Message::Move(i, false)),
                        ))
                        .push(widget::button::text("×").on_press(Message::Remove(i))),
                );
            }
            body = body.push(widget::scrollable(list).height(130));
            body = body.push(
                widget::flex_row(
                    Transform::ALL
                        .iter()
                        .map(|m| {
                            widget::button::text(m.label())
                                .class(crate::skin::button(self.mode == *m, 8.0, false))
                                .on_press(Message::Mode(*m))
                                .into()
                        })
                        .collect(),
                )
                .spacing(4),
            );
            let labels = [
                tr!("Sans séparateur", "No separator"),
                tr!("Retour à la ligne", "Newline"),
                tr!("Ligne vide", "Blank line"),
                tr!("Virgule", "Comma"),
                tr!("Personnalisé", "Custom"),
            ];
            body = body.push(
                widget::flex_row(
                    labels
                        .iter()
                        .enumerate()
                        .map(|(i, l)| {
                            widget::button::text(*l)
                                .class(crate::skin::button(self.separator == i as u8, 8.0, false))
                                .on_press(Message::Separator(i as u8))
                                .into()
                        })
                        .collect(),
                )
                .spacing(4),
            );
            if self.separator == 4 {
                body = body.push(
                    widget::text_input(tr!("Séparateur", "Separator"), &self.custom)
                        .on_input(Message::Custom),
                );
            }
            let result = self.result();
            let valid = result.is_ok() && !self.busy;
            body = body
                .push(
                    widget::text(tr!(
                        "Aperçu du résultat — originaux conservés",
                        "Result preview — originals preserved"
                    ))
                    .size(13),
                )
                .push(
                    widget::scrollable(widget::text(result.unwrap_or_else(|e| e)).size(13))
                        .height(150),
                );
            body = body.push(
                widget::flex_row(vec![
                    widget::button::suggested(tr!("Copier le résultat", "Copy result"))
                        .on_press_maybe(valid.then_some(Message::Copy))
                        .into(),
                    widget::button::text(tr!("Enregistrer comme note…", "Save as note…"))
                        .on_press_maybe(valid.then_some(Message::SaveNote))
                        .into(),
                    widget::button::text(tr!("Préparer la file", "Prepare queue"))
                        .on_press_maybe(valid.then_some(Message::MakeQueue))
                        .into(),
                ])
                .spacing(4),
            );
        }
        body = body.push(
            widget::text(tr_format!(
                "File de copie : {} / {} effectuées",
                "Copy queue: {} / {} completed",
                self.queue.next,
                self.queue.parts.len()
            ))
            .size(16),
        );
        if let Some(p) = self.queue.current() {
            body =
                body.push(widget::text(tr_format!("Suivant : {}", "Next: {}", p.title)).size(12));
        }
        body = body.push(
            widget::flex_row(vec![
                widget::button::text(tr!("Reculer", "Previous"))
                    .on_press_maybe(
                        (!self.busy && self.queue.next > 0).then_some(Message::Previous),
                    )
                    .into(),
                widget::button::suggested(tr!("Copier le suivant", "Copy next"))
                    .on_press_maybe(
                        (!self.busy && self.queue.current().is_some()).then_some(Message::Next),
                    )
                    .into(),
                widget::button::text(tr!("Vider la file", "Clear queue"))
                    .on_press_maybe((!self.busy).then_some(Message::ClearQueue))
                    .into(),
            ])
            .spacing(4),
        );
        body = body.push(
            widget::text(tr!(
                "Favoris rapides — cliquer « Changer » parcourt tes favoris",
                "Quick favorites — Change cycles through your favorites"
            ))
            .size(16),
        );
        for i in 0..5 {
            let title = prefs.slots[i]
                .as_ref()
                .and_then(|id| clips.iter().find(|c| &c.id == id && c.pinned))
                .map(|c| c.title.chars().take(35).collect::<String>())
                .unwrap_or_else(|| tr!("Vide ou indisponible", "Empty or unavailable").into());
            body = body.push(
                widget::flex_row(vec![
                    widget::text(format!("{} · {title}", i + 1)).into(),
                    widget::button::text(tr!("Copier", "Copy"))
                        .on_press(Message::Favorite(i))
                        .into(),
                    widget::button::text(tr!("Changer", "Change"))
                        .on_press(Message::Slot(i))
                        .into(),
                    widget::button::text(tr!("Effacer", "Clear"))
                        .on_press(Message::ClearSlot(i))
                        .into(),
                ])
                .spacing(4),
            );
        }
        body = body
            .push(widget::text(tr!("Action au clic sur une copie", "Clip click action")).size(16));
        for (i, label) in [
            tr!("Compacte", "Compact"),
            tr!("Bandeau", "Shelf"),
            tr!("Fenêtre", "Window"),
        ]
        .iter()
        .enumerate()
        {
            body = body.push(
                widget::button::text(format!(
                    "{label} : {}",
                    [
                        tr!("Copier", "Copy"),
                        tr!("Aperçu", "Preview"),
                        tr!("Sélectionner", "Select")
                    ][prefs.clicks[i] as usize]
                ))
                .on_press(Message::Click(i)),
            );
        }
        body=body.push(widget::text(tr!("Raccourcis dans Nebula","Shortcuts inside Nebula")).size(16)).push(widget::text(tr!("Laisse vide pour désactiver. Les raccourcis système se configurent dans COSMIC ; voir le guide 1.3.","Leave empty to disable. Configure global shortcuts in COSMIC; see the 1.3 guide.")).size(12));
        for (i, d) in self.drafts.iter().enumerate() {
            body = body.push(
                widget::column([])
                    .spacing(3)
                    .push(widget::text(text::shortcut_label(i)).size(12))
                    .push(
                        widget::row([])
                            .spacing(4)
                            .push(
                                widget::text_input("Ctrl+Shift+K", d)
                                    .on_input(move |s| Message::Binding(i, s)),
                            )
                            .push(
                                widget::button::text(tr!("Appliquer", "Apply"))
                                    .on_press(Message::SaveBinding(i)),
                            ),
                    ),
            );
        }
        body.push(
            widget::button::text(tr!("Réinitialiser les raccourcis", "Reset shortcuts"))
                .on_press(Message::ResetBindings),
        )
        .into()
    }
}
pub fn clip_part(clip: &Clip) -> Option<Part> {
    if matches!(clip.kind, Kind::Image | Kind::Files) {
        None
    } else {
        Some(Part {
            title: clip.title.clone(),
            text: clip.text.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queue_is_frozen_and_edits_are_blocked_during_copy() {
        let mut state = State::default();
        state
            .load(vec![
                Part {
                    title: "A".into(),
                    text: "a".into(),
                },
                Part {
                    title: "B".into(),
                    text: "b".into(),
                },
            ])
            .unwrap();
        let mut prefs = Preferences::default();
        state.update(Message::MakeQueue, &mut prefs, &[]).unwrap();
        state
            .update(Message::Move(1, true), &mut prefs, &[])
            .unwrap();
        assert_eq!(state.result().unwrap(), "b\na");
        assert_eq!(state.queue.current().unwrap().text, "a");
        state.busy = true;
        assert!(state.update(Message::MakeQueue, &mut prefs, &[]).is_err());
        assert!(state.load(vec![]).is_err());
        assert_eq!(state.queue.next, 0);
    }
    #[test]
    fn favorite_slot_never_falls_back_to_another_clip() {
        let mut state = State::default();
        let mut p = Preferences::default();
        p.slots[0] = Some("deleted".into());
        let mut c = Clip::new("text/plain".into(), b"other".to_vec(), 1).unwrap();
        c.pinned = true;
        assert!(
            state
                .update(Message::Favorite(0), &mut p, &[c.clone()])
                .is_err()
        );
        state
            .update(Message::Slot(0), &mut p, &[c.clone()])
            .unwrap();
        match state
            .update(Message::Favorite(0), &mut p, &[c.clone()])
            .unwrap()
        {
            Effect::Favorite(id) => assert_eq!(id, c.id),
            _ => panic!("wrong effect"),
        };
    }
    #[test]
    fn transformed_result_can_be_saved_without_mutating_source() {
        let mut state = State::default();
        let original = Part {
            title: "A".into(),
            text: "Été {{x}}".into(),
        };
        state.load(vec![original.clone()]).unwrap();
        let mut p = Preferences::default();
        state
            .update(Message::Mode(Transform::Upper), &mut p, &[])
            .unwrap();
        match state.update(Message::SaveNote, &mut p, &[]).unwrap() {
            Effect::Note(s) => assert_eq!(s, "ÉTÉ {{X}}"),
            _ => panic!("note expected"),
        };
        assert_eq!(state.parts[0], original);
    }
}
