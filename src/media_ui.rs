//! Explicit image workbench: previews are drafts until the user saves/copies.
use crate::capture::Capture;
use cosmic::{
    Element,
    iced::{self, Length},
    widget,
};
use nebula_paste::{
    imaging::{Format, Plan},
    model::{self, Clip, Kind},
    tr, tr_format,
};
use std::sync::Arc;
#[derive(Debug, Clone)]
pub struct Prepared {
    pub clip: Arc<Clip>,
    pub thumbnail: Option<iced::widget::image::Handle>,
    pub dimensions: [u32; 2],
}
pub fn prepare(clip: Arc<Clip>) -> Result<Prepared, String> {
    if clip.kind != Kind::Image {
        return Ok(Prepared {
            clip,
            thumbnail: None,
            dimensions: [0, 0],
        });
    }
    let decoded = model::decode_image(&clip.bytes)?;
    let dimensions = [decoded.width(), decoded.height()];
    let small = decoded.thumbnail(720, 420).to_rgba8();
    let thumbnail = Some(iced::widget::image::Handle::from_rgba(
        small.width(),
        small.height(),
        small.into_raw(),
    ));
    Ok(Prepared {
        clip,
        thumbnail,
        dimensions,
    })
}
#[derive(Debug, Clone)]
pub enum Message {
    Capture(Capture),
    Import,
    Loaded(Result<Option<Prepared>, String>, bool),
    Crop(usize, String),
    Size(usize, String),
    Aspect,
    Format(Format),
    Quality(u8),
    Preview,
    Rendered(Result<Prepared, String>),
    Reset,
    Ocr,
    OcrDone(Result<String, String>),
    EditText(widget::text_editor::Action),
    CopyImage,
    SaveImage,
    Export,
    Exported(Result<bool, String>),
    CopyText,
    Note,
    SaveColor,
    CopyColor(bool),
    Copied(Result<(), String>),
    Clear,
    ConfirmClear,
    CancelClear,
}
pub struct State {
    pub source: Option<Prepared>,
    pub output: Option<Prepared>,
    pub crop: [String; 4],
    pub size: [String; 2],
    pub aspect: bool,
    pub format: Format,
    pub quality: u8,
    pub dirty: bool,
    pub busy: bool,
    pub text: widget::text_editor::Content,
    pub text_ready: bool,
    pub notice: String,
    pub confirm_clear: bool,
    pub return_popup: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            source: None,
            output: None,
            crop: Default::default(),
            size: Default::default(),
            aspect: true,
            format: Format::Png,
            quality: 85,
            dirty: false,
            busy: false,
            text: widget::text_editor::Content::new(),
            text_ready: false,
            notice: String::new(),
            confirm_clear: false,
            return_popup: true,
        }
    }
}
impl State {
    pub fn occupied(&self) -> bool {
        self.source.is_some() || !self.text.text().is_empty()
    }
    pub fn load(&mut self, p: Prepared) {
        self.source = Some(p);
        self.text = widget::text_editor::Content::new();
        self.text_ready = false;
        self.reset();
    }
    pub fn reset(&mut self) {
        if let Some(s) = &self.source {
            self.crop = [
                "0".into(),
                "0".into(),
                s.dimensions[0].to_string(),
                s.dimensions[1].to_string(),
            ];
            self.size = s.dimensions.map(|n| n.to_string());
            self.format = match s.clip.mime.as_str() {
                "image/jpeg" => Format::Jpeg,
                "image/webp" => Format::Webp,
                _ => Format::Png,
            };
        }
        self.output = None;
        self.dirty = false;
        self.aspect = true;
        self.quality = 85;
    }
    pub fn changed(&mut self) {
        self.output = None;
        self.dirty = true;
    }
    pub fn set_size(&mut self, i: usize, s: String) {
        if i >= 2 || s.len() > 8 {
            return;
        }
        self.size[i] = s;
        if self.aspect {
            if let (Ok(n), Ok(w), Ok(h)) = (
                self.size[i].parse::<u32>(),
                self.crop[2].parse::<u32>(),
                self.crop[3].parse::<u32>(),
            ) {
                let (a, b) = if i == 0 { (w, h) } else { (h, w) };
                if a > 0 {
                    self.size[1 - i] = ((u64::from(n) * u64::from(b) + u64::from(a) / 2)
                        / u64::from(a))
                    .max(1)
                    .to_string();
                }
            }
        }
        self.changed();
    }
    pub fn plan(&self) -> Result<Plan, String> {
        let number = |s: &String| {
            s.parse::<u32>().map_err(|_| {
                tr!(
                    "Saisis des dimensions entières positives.",
                    "Enter positive whole-number dimensions."
                )
                .to_string()
            })
        };
        let p = Plan {
            crop: [
                number(&self.crop[0])?,
                number(&self.crop[1])?,
                number(&self.crop[2])?,
                number(&self.crop[3])?,
            ],
            size: [number(&self.size[0])?, number(&self.size[1])?],
            format: self.format,
            quality: self.quality,
        };
        p.validate(self.source.as_ref().ok_or("No source image")?.dimensions)?;
        Ok(p)
    }
    pub fn result(&self) -> Option<Arc<Clip>> {
        if self.dirty {
            None
        } else {
            self.output
                .as_ref()
                .or(self.source.as_ref())
                .map(|p| p.clip.clone())
        }
    }
    pub fn editable_text(&self) -> Result<String, String> {
        let s = self.text.text();
        nebula_paste::text_actions::check(&s)?;
        if s.trim().is_empty() {
            return Err(tr!("Aucun texte à copier.", "No text to copy.").into());
        }
        Ok(s)
    }
    pub fn view(&self) -> Element<'_, Message> {
        let idle = !self.busy;
        let empty = idle && !self.occupied();
        let mut body=widget::column([]).spacing(10)
            .push(widget::text(tr!("Capture et images","Capture and images")).size(20))
            .push(widget::text(tr!("Choisis une zone dans le dialogue du bureau. Les résultats restent dans cet atelier jusqu’à ton action.","Choose an area in the desktop dialog. Results stay in this workbench until you act.")).size(12))
            .push(widget::flex_row(vec![
                widget::button::standard(tr!("Capturer…","Capture…")).on_press_maybe(empty.then_some(Message::Capture(Capture::Image))).into(),
                widget::button::standard(tr!("Texte de l’écran…","Screen text…")).on_press_maybe(empty.then_some(Message::Capture(Capture::Text))).into(),
                widget::button::standard(tr!("Pipette…","Pick color…")).on_press_maybe(empty.then_some(Message::Capture(Capture::Color))).into(),
                widget::button::text(tr!("Importer une image…","Import image…")).on_press_maybe(empty.then_some(Message::Import)).into(),
            ]).spacing(6));
        if self.busy {
            body = body.push(widget::text(tr!("Traitement en cours…", "Working…")));
        }
        if !self.notice.is_empty() {
            body = body.push(widget::text(&self.notice).size(13));
        }
        if self.text_ready {
            body = body.push(
                widget::text_editor(&self.text)
                    .height(150)
                    .on_action(Message::EditText),
            );
            let text_ready = idle && self.editable_text().is_ok();
            body = body.push(
                widget::flex_row(vec![
                    widget::button::standard(tr!("Copier le texte", "Copy text"))
                        .on_press_maybe(text_ready.then_some(Message::CopyText))
                        .into(),
                    widget::button::text(tr!("Créer une note…", "Create note…"))
                        .on_press_maybe(text_ready.then_some(Message::Note))
                        .into(),
                ])
                .spacing(6),
            );
        }
        if let Some(source) = &self.source {
            if source.clip.kind == Kind::Image {
                let shown = self.output.as_ref().unwrap_or(source);
                if let Some(handle) = &shown.thumbnail {
                    body = body.push(
                        widget::image(handle.clone())
                            .width(Length::Fill)
                            .height(200)
                            .content_fit(iced::ContentFit::Contain),
                    );
                }
                body = body.push(
                    widget::text(tr_format!(
                        "Original : {} × {} · Aperçu : {} × {}",
                        "Original: {} × {} · Preview: {} × {}",
                        source.dimensions[0],
                        source.dimensions[1],
                        shown.dimensions[0],
                        shown.dimensions[1]
                    ))
                    .size(12),
                );
                if self.dirty {
                    body = body.push(
                        widget::text(tr!(
                            "Réglages modifiés : calcule l’aperçu avant de copier ou d’exporter.",
                            "Settings changed: build the preview before copying or exporting."
                        ))
                        .size(12),
                    );
                }
                let mut crop = Vec::new();
                for (i, label) in ["X", "Y", tr!("Largeur", "Width"), tr!("Hauteur", "Height")]
                    .into_iter()
                    .enumerate()
                {
                    crop.push(
                        widget::column([])
                            .push(widget::text(label).size(11))
                            .push(
                                widget::text_input("0", &self.crop[i])
                                    .on_input(move |s| Message::Crop(i, s)),
                            )
                            .width(125)
                            .into(),
                    );
                }
                body = body
                    .push(
                        widget::text(tr!(
                            "Recadrer · pixels de l’original",
                            "Crop · original-image pixels"
                        ))
                        .size(15),
                    )
                    .push(widget::flex_row(crop).spacing(6));
                let mut resize = Vec::new();
                for (i, label) in [
                    tr!("Largeur finale", "Output width"),
                    tr!("Hauteur finale", "Output height"),
                ]
                .into_iter()
                .enumerate()
                {
                    resize.push(
                        widget::column([])
                            .push(widget::text(label).size(11))
                            .push(
                                widget::text_input("", &self.size[i])
                                    .on_input(move |s| Message::Size(i, s)),
                            )
                            .width(150)
                            .into(),
                    );
                }
                body = body.push(widget::flex_row(resize).spacing(6)).push(
                    widget::button::text(if self.aspect {
                        tr!("Proportions liées", "Aspect ratio locked")
                    } else {
                        tr!("Proportions libres", "Free aspect ratio")
                    })
                    .on_press_maybe(idle.then_some(Message::Aspect)),
                );
                body = body.push(
                    widget::flex_row(
                        Format::ALL
                            .into_iter()
                            .map(|f| {
                                widget::button::text(f.label())
                                    .class(crate::skin::button(self.format == f, 8.0, false))
                                    .on_press_maybe(idle.then_some(Message::Format(f)))
                                    .into()
                            })
                            .collect(),
                    )
                    .spacing(6),
                );
                if self.format == Format::Jpeg {
                    body = body.push(
                        widget::text(tr!(
                            "JPEG : transparence remplacée par du blanc.",
                            "JPEG: transparency is flattened onto white."
                        ))
                        .size(12),
                    );
                    body = body.push(
                        widget::flex_row(
                            [70, 85, 95]
                                .into_iter()
                                .map(|q| {
                                    widget::button::text(format!("JPEG {q}%"))
                                        .class(crate::skin::button(self.quality == q, 8.0, false))
                                        .on_press_maybe(idle.then_some(Message::Quality(q)))
                                        .into()
                                })
                                .collect(),
                        )
                        .spacing(5),
                    );
                }
                body = body.push(
                    widget::flex_row(vec![
                        widget::button::suggested(tr!("Calculer l’aperçu", "Build preview"))
                            .on_press_maybe(idle.then_some(Message::Preview))
                            .into(),
                        widget::button::text(tr!("Réinitialiser", "Reset"))
                            .on_press_maybe(idle.then_some(Message::Reset))
                            .into(),
                    ])
                    .spacing(6),
                );
                let ready = idle && !self.dirty;
                body = body.push(
                    widget::flex_row(vec![
                        widget::button::standard(tr!("Copier l’image", "Copy image"))
                            .on_press_maybe(ready.then_some(Message::CopyImage))
                            .into(),
                        widget::button::text(tr!("Ajouter à l’historique", "Add to history"))
                            .on_press_maybe(ready.then_some(Message::SaveImage))
                            .into(),
                        widget::button::text(tr!("Exporter…", "Export…"))
                            .on_press_maybe(ready.then_some(Message::Export))
                            .into(),
                        widget::button::text(tr!("Extraire le texte", "Extract text"))
                            .on_press_maybe(ready.then_some(Message::Ocr))
                            .into(),
                    ])
                    .spacing(6),
                );
                body=body.push(widget::text(tr!("OCR local · langue choisie dans les préférences. Export vers un nouveau fichier.","Local OCR · language selected in preferences. Export to a new file.")).size(11));
            } else if let Some(rgb) = model::color(&source.clip.text) {
                body = body
                    .push(
                        widget::container(widget::Space::new().height(100))
                            .width(Length::Fill)
                            .class(crate::skin::swatch(iced::Color::from_rgb8(
                                rgb[0], rgb[1], rgb[2],
                            ))),
                    )
                    .push(widget::text(format!(
                        "{} · rgb({}, {}, {})",
                        source.clip.text, rgb[0], rgb[1], rgb[2]
                    )))
                    .push(
                        widget::flex_row(vec![
                            widget::button::standard("HEX")
                                .on_press_maybe(idle.then_some(Message::CopyColor(false)))
                                .into(),
                            widget::button::standard("RGB")
                                .on_press_maybe(idle.then_some(Message::CopyColor(true)))
                                .into(),
                            widget::button::text(tr!("Enregistrer la couleur", "Save color"))
                                .on_press_maybe(idle.then_some(Message::SaveColor))
                                .into(),
                        ])
                        .spacing(6),
                    );
            }
            body = body.push(
                widget::button::text(tr!("Effacer l’atelier…", "Clear workbench…"))
                    .on_press_maybe(idle.then_some(Message::Clear)),
            );
        }
        if self.confirm_clear {
            body = body
                .push(widget::text(tr!(
                    "Abandonner l’aperçu et le texte non enregistrés ?",
                    "Discard the unsaved preview and text?"
                )))
                .push(
                    widget::row([])
                        .spacing(6)
                        .push(
                            widget::button::destructive(tr!("Abandonner", "Discard"))
                                .on_press(Message::ConfirmClear),
                        )
                        .push(
                            widget::button::text(tr!("Garder", "Keep"))
                                .on_press(Message::CancelClear),
                        ),
                );
        }
        body.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changing_settings_disables_stale_output_and_reset_restores_original() {
        let clip = crate::demo::clips()
            .into_iter()
            .find(|c| c.kind == Kind::Image)
            .unwrap();
        let mut state = State::default();
        state.load(prepare(Arc::new(clip)).unwrap());
        assert!(state.result().is_some());
        state.set_size(0, "160".into());
        assert_eq!(state.size, ["160", "90"]);
        assert!(state.result().is_none());
        state.reset();
        assert_eq!(state.size, ["320", "180"]);
        assert!(state.result().is_some());
    }
    #[test]
    fn empty_ocr_is_not_copyable() {
        assert!(State::default().editable_text().is_err());
    }
}
