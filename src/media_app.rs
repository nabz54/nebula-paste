use super::*;
use crate::{
    capture::Capture,
    media_ui::{self, Message as M},
};
use cosmic::Application;
use std::sync::Arc;
impl App {
    fn show_media(&mut self) -> Task<cosmic::Action<Message>> {
        let task = if !self.demo && self.popup.is_none() && self.history.is_none() {
            if self.media.return_popup {
                self.update(Message::Toggle)
            } else {
                self.update(Message::OpenHistory)
            }
        } else {
            Task::none()
        };
        self.media_open = true;
        self.actions_open = false;
        self.notes_open = false;
        self.templates_open = false;
        self.workspace_open = false;
        self.settings_open = false;
        self.collections_open = false;
        self.detail = None;
        self.pause_menu = false;
        self.clear_confirm = false;
        task
    }
    pub(super) fn open_media(&mut self) -> Task<cosmic::Action<Message>> {
        self.show_media()
    }
    pub(super) fn media_from_clip(&mut self, id: String) -> Task<cosmic::Action<Message>> {
        if self.media.busy {
            return Task::none();
        }
        if self.media.occupied() {
            self.media.notice=tr!("Un atelier est déjà ouvert. Efface-le explicitement avant de charger une autre image.","A workbench is already open. Clear it explicitly before loading another image.").into();
            return self.show_media();
        }
        let Some(clip) = self
            .clips
            .iter()
            .find(|c| c.id == id && c.kind == Kind::Image)
            .cloned()
        else {
            return Task::none();
        };
        self.media.busy = true;
        self.show_media().chain(Task::perform(
            async move {
                tokio::task::spawn_blocking(move || media_ui::prepare(Arc::new(clip)))
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r)
                    .map(Some)
            },
            |r| cosmic::Action::App(Message::Media(M::Loaded(r, false))),
        ))
    }
    fn media_copy(&mut self, clip: Arc<Clip>) -> Task<cosmic::Action<Message>> {
        if self.copying {
            self.media.notice = tr!(
                "Une copie est déjà en cours.",
                "Another copy is in progress."
            )
            .into();
            return Task::none();
        }
        if self.demo {
            self.media.notice = tr!(
                "Démonstration : presse-papiers inchangé.",
                "Demo: clipboard unchanged."
            )
            .into();
            return Task::none();
        }
        self.copying = true;
        self.media.busy = true;
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || clipboard::copy((*clip).clone()))
                    .await
                    .map_err(|e| e.to_string())
                    .and_then(|r| r)
            },
            |r| cosmic::Action::App(Message::Media(M::Copied(r))),
        )
    }
    pub(super) fn update_media(&mut self, msg: M) -> Task<cosmic::Action<Message>> {
        if self.media.busy
            && !matches!(
                &msg,
                M::Loaded(..) | M::Rendered(_) | M::OcrDone(_) | M::Exported(_) | M::Copied(_)
            )
        {
            return Task::none();
        }
        match msg {
            M::Capture(kind) => {
                if self.media.occupied() {
                    self.media.notice = tr!(
                        "Efface d’abord l’atelier pour une nouvelle capture.",
                        "Clear the workbench before a new capture."
                    )
                    .into();
                    return self.show_media();
                }
                if self.demo {
                    self.media.notice = tr!(
                        "Capture du bureau désactivée dans la démonstration.",
                        "Desktop capture disabled in the demo."
                    )
                    .into();
                    return self.show_media();
                }
                self.media.busy = true;
                self.media.notice.clear();
                self.media.return_popup = self.popup.is_some() || self.history.is_none();
                let close_popup = self.popup.take().map_or_else(Task::none, destroy_popup);
                let close_window = self
                    .history
                    .take()
                    .map_or_else(Task::none, iced::window::close);
                return Task::batch([close_popup, close_window]).chain(Task::perform(
                    async move {
                        // Let closed surfaces disappear before the portal captures the desktop.
                        tokio::time::sleep(Duration::from_millis(200)).await;
                        let result = crate::capture::portal(kind).await?;
                        match result {
                            Some(c) => {
                                tokio::task::spawn_blocking(move || media_ui::prepare(c).map(Some))
                                    .await
                                    .map_err(|e| e.to_string())?
                            }
                            None => Ok(None),
                        }
                    },
                    move |r| {
                        cosmic::Action::App(Message::Media(M::Loaded(r, kind == Capture::Text)))
                    },
                ));
            }
            M::Import => {
                if self.media.occupied() {
                    self.media.notice =
                        tr!("Efface d’abord l’atelier.", "Clear the workbench first.").into();
                    return Task::none();
                }
                if self.demo {
                    return Task::none();
                }
                self.media.busy = true;
                self.media.return_popup = self.popup.is_some() || self.history.is_none();
                return Task::perform(
                    async {
                        match crate::capture::import().await? {
                            Some(c) => {
                                tokio::task::spawn_blocking(move || media_ui::prepare(c).map(Some))
                                    .await
                                    .map_err(|e| e.to_string())?
                            }
                            None => Ok(None),
                        }
                    },
                    |r| cosmic::Action::App(Message::Media(M::Loaded(r, false))),
                );
            }
            M::Loaded(result, ocr) => {
                self.media.busy = false;
                match result {
                    Ok(Some(p)) => {
                        self.media.load(p);
                        self.media.notice = tr!(
                            "Aperçu prêt · rien n’a encore été enregistré.",
                            "Preview ready · nothing has been saved yet."
                        )
                        .into();
                        let show = self.show_media();
                        if ocr {
                            return show.chain(self.update_media(M::Ocr));
                        }
                        return show;
                    }
                    Ok(None) => {
                        self.media.notice =
                            tr!("Capture ou import annulé.", "Capture or import cancelled.").into()
                    }
                    Err(e) => self.media.notice = e,
                }
                return self.show_media();
            }
            M::Crop(i, s) => {
                if i < 4 && s.len() <= 8 {
                    self.media.crop[i] = s;
                    if i >= 2 {
                        self.media.size = [self.media.crop[2].clone(), self.media.crop[3].clone()];
                    }
                    self.media.changed();
                }
            }
            M::Size(i, s) => self.media.set_size(i, s),
            M::Aspect => {
                self.media.aspect = !self.media.aspect;
                if self.media.aspect {
                    self.media.set_size(0, self.media.size[0].clone());
                }
            }
            M::Format(f) => {
                self.media.format = f;
                self.media.changed();
            }
            M::Quality(q) => {
                self.media.quality = q;
                self.media.changed();
            }
            M::Reset => self.media.reset(),
            M::Preview => {
                let plan = match self.media.plan() {
                    Ok(p) => p,
                    Err(e) => {
                        self.media.notice = e;
                        return Task::none();
                    }
                };
                let Some(source) = self.media.source.as_ref().map(|s| s.clip.clone()) else {
                    return Task::none();
                };
                self.media.busy = true;
                self.media.notice.clear();
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            nebula_paste::imaging::transform(&source.bytes, plan)
                                .and_then(|c| media_ui::prepare(Arc::new(c)))
                        })
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r)
                    },
                    |r| cosmic::Action::App(Message::Media(M::Rendered(r))),
                );
            }
            M::Rendered(r) => {
                self.media.busy = false;
                match r {
                    Ok(p) => {
                        self.media.output = Some(p);
                        self.media.dirty = false;
                        self.media.notice = tr!(
                            "Aperçu calculé · original conservé.",
                            "Preview built · original preserved."
                        )
                        .into();
                    }
                    Err(e) => self.media.notice = e,
                }
            }
            M::Ocr => {
                if self.ocr_busy || self.index_busy.is_some() {
                    self.media.notice = tr!(
                        "OCR occupé. Réessaie dans un instant.",
                        "OCR busy. Try again shortly."
                    )
                    .into();
                    return Task::none();
                }
                if !self.media.text.text().trim().is_empty() {
                    self.media.notice = tr!(
                        "Efface le texte de l’atelier avant de relancer l’OCR.",
                        "Clear the workbench text before running OCR again."
                    )
                    .into();
                    return Task::none();
                }
                let Some(clip) = self.media.result().filter(|c| c.kind == Kind::Image) else {
                    return Task::none();
                };
                self.ocr_busy = true;
                self.media.busy = true;
                self.media.notice.clear();
                let language = self.settings.ocr_language;
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            crate::actions::ocr((*clip).clone(), language)
                        })
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r)
                    },
                    |r| cosmic::Action::App(Message::Media(M::OcrDone(r))),
                );
            }
            M::OcrDone(r) => {
                self.ocr_busy = false;
                self.media.busy = false;
                self.media.text_ready = true;
                match r.and_then(|s| nebula_paste::text_actions::check(&s).map(|_| s)) {
                    Ok(s) => {
                        self.media.notice = if s.trim().is_empty() {
                            tr!("Aucun texte détecté.", "No text detected.")
                        } else {
                            tr!(
                                "Vérifie et corrige le texte avant de le copier.",
                                "Review and correct the text before copying."
                            )
                        }
                        .into();
                        self.media.text = widget::text_editor::Content::with_text(&s);
                    }
                    Err(e) => self.media.notice = e,
                }
            }
            M::EditText(action) => {
                let previous = self.media.text.text();
                self.media.text.perform(action);
                if nebula_paste::text_actions::check(&self.media.text.text()).is_err() {
                    self.media.text = widget::text_editor::Content::with_text(&previous);
                    self.media.notice =
                        tr!("Texte limité à 256 Kio.", "Text limited to 256 KiB.").into();
                }
            }
            M::CopyImage => {
                if let Some(c) = self.media.result() {
                    return self.media_copy(c);
                }
            }
            M::CopyColor(rgb) => {
                if let Some(c) = self.media.result() {
                    if let Some([r, g, b]) = model::color(&c.text) {
                        let text = if rgb {
                            format!("rgb({r}, {g}, {b})")
                        } else {
                            c.text.clone()
                        };
                        if let Ok(c) = Clip::new(
                            "text/plain;charset=utf-8".into(),
                            text.into_bytes(),
                            model::now(),
                        ) {
                            return self.media_copy(Arc::new(c));
                        }
                    }
                }
            }
            M::SaveImage | M::SaveColor => {
                if let Some(c) = self.media.result() {
                    match self
                        .store
                        .as_mut()
                        .ok_or(tr!("Historique indisponible.", "History unavailable.").to_string())
                        .and_then(|s| s.insert(&c).map_err(|e| e.to_string()))
                    {
                        Ok(()) => {
                            self.refresh();
                            self.media.notice =
                                tr!("Ajouté à l’historique.", "Added to history.").into();
                        }
                        Err(e) => self.media.notice = e,
                    }
                }
            }
            M::Export => {
                if self.demo {
                    return Task::none();
                }
                if let Some(c) = self.media.result() {
                    self.media.busy = true;
                    return Task::perform(crate::capture::export(c), |r| {
                        cosmic::Action::App(Message::Media(M::Exported(r)))
                    });
                }
            }
            M::Exported(r) => {
                self.media.busy = false;
                self.media.notice = match r {
                    Ok(true) => tr!("Image exportée.", "Image exported.").into(),
                    Ok(false) => tr!("Export annulé.", "Export cancelled.").into(),
                    Err(e) => e,
                };
            }
            M::Copied(r) => {
                self.media.busy = false;
                self.copying = false;
                self.media.notice = match r {
                    Ok(()) => {
                        tr!("Copié · colle avec Ctrl+V.", "Copied · paste with Ctrl+V.").into()
                    }
                    Err(e) => e,
                };
            }
            M::CopyText | M::Note => {
                let text = match self.media.editable_text() {
                    Ok(s) => s,
                    Err(e) => {
                        self.media.notice = e;
                        return Task::none();
                    }
                };
                if matches!(msg, M::Note) {
                    if text.len() > nebula_paste::notes::MAX_BODY {
                        self.media.notice = tr!(
                            "Une note est limitée à 64 Kio.",
                            "A note is limited to 64 KiB."
                        )
                        .into();
                        return Task::none();
                    }
                    self.notes
                        .from_clip(tr!("Texte capturé", "Captured text"), &text, "");
                    return self.update(Message::Notes(true));
                }
                if let Ok(c) = Clip::new(
                    "text/plain;charset=utf-8".into(),
                    text.into_bytes(),
                    model::now(),
                ) {
                    return self.media_copy(Arc::new(c));
                }
            }
            M::Clear => self.media.confirm_clear = true,
            M::CancelClear => self.media.confirm_clear = false,
            M::ConfirmClear => self.media = media_ui::State::default(),
        }
        Task::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic::Application;
    #[test]
    fn capture_and_ocr_results_are_drafts_until_explicit_save() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        let before = app.clips.len();
        let clip = crate::demo::clips()
            .into_iter()
            .find(|c| c.kind == Kind::Image)
            .unwrap();
        let _ = app.update_media(M::Loaded(
            Ok(Some(media_ui::prepare(Arc::new(clip)).unwrap())),
            false,
        ));
        app.media.busy = true;
        app.ocr_busy = true;
        let _ = app.update_media(M::OcrDone(Ok("draft text".into())));
        assert_eq!(app.clips.len(), before);
        assert_eq!(app.media.text.text(), "draft text");
        assert!(!app.media.busy);
        assert!(!app.ocr_busy);
        let _ = app.update_media(M::Clear);
        assert!(app.media.source.is_some());
        let _ = app.update_media(M::ConfirmClear);
        assert!(app.media.source.is_none());
    }
    #[test]
    fn busy_workbench_cannot_be_reset_and_cancel_recovers() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        app.media.busy = true;
        let _ = app.update_media(M::ConfirmClear);
        assert!(app.media.busy);
        let _ = app.update_media(M::Loaded(Ok(None), false));
        assert!(!app.media.busy);
        assert!(app.media_open);
    }
}
