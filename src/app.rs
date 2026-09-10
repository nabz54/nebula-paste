use crate::{ipc::Instance, skin};
use cosmic::{
    iced::{
        self, Color, Length, Limits, Subscription,
        keyboard::{self, Key, key::Named},
        platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup},
        window::Id,
    },
    prelude::*,
    widget,
};
use nebula_paste::{
    clipboard::{self, Event, Monitor},
    model::{self, Clip, Kind},
    settings::{Density, Settings},
    storage::Store,
};
use nebula_paste::{tr, tr_format};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

/// Bornes de largeur du volet. La largeur réelle vient du compositeur : le
/// positionneur accepte désormais un volet plus étroit que la valeur idéale.
const WIDTH_WIDE: f32 = 940.0;
const WIDTH_COMPACT: f32 = 470.0;
const WIDTH_MIN: f32 = 360.0;
/// Largeur minimale d’une carte de la grille et espacement entre les cartes.
const CARD_WIDTH: f32 = 205.0;
const CARD_SPACING: f32 = 12.0;
/// Marge horizontale totale du volet.
const PADDING: f32 = 36.0;
const MAX_COLUMNS: usize = 4;
/// Nombre maximal de cartes accessibles par Ctrl+chiffre.
const MAX_SHORTCUTS: usize = 9;
const UNDO_SECONDS: u64 = 12;
const FLASH_SECONDS: u64 = 4;
/// Durées proposées par la pause temporaire, en minutes.
const PAUSES: [u64; 3] = [5, 15, 60];
pub enum Mode {
    Applet(Instance),
    Preview,
}
pub struct App {
    core: cosmic::Core,
    popup: Option<Id>,
    instance: Option<Instance>,
    demo: bool,
    monitor: Monitor,
    store: Option<Store>,
    clips: Vec<Clip>,
    thumbnails: HashMap<String, iced::widget::image::Handle>,
    payloads: HashMap<String, crate::transfer::Payload>,
    query: String,
    kind: Option<Kind>,
    favorites: bool,
    category: String,
    category_edit: String,
    detail: Option<String>,
    page: usize,
    selected: usize,
    status: String,
    connected: bool,
    clear_confirm: bool,
    copying: bool,
    search_id: iced::widget::Id,
    ocr_busy: bool,
    dragging: bool,
    settings: Settings,
    settings_path: Option<PathBuf>,
    settings_open: bool,
    retention_draft: u32,
    pause_menu: bool,
    /// Fin de la pause temporaire, en secondes Unix. `None` : pause sans limite.
    pause_until: Option<Instant>,
    /// Dernière largeur utile annoncée par le compositeur.
    viewport: f32,
    undo: Option<Undo>,
    /// Confirmation visible, remplaçant l’état courant jusqu’à son échéance.
    flash: Option<(String, Instant)>,
}

/// Suppression annulable : le contenu est conservé en mémoire, jamais réécrit
/// sur disque tant que l’annulation n’est pas demandée.
struct Undo {
    clip: Clip,
    until: Instant,
}

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
    PasteMode,
    Ocr(String),
    OcrLanguage,
    OcrDone(String, Result<String, String>),
    Pasted(Result<(), String>),
    PasteReady(bool),
    Drag(bool),
    Closed(Id),
    Tick,
    Search(String),
    Filter(Option<Kind>),
    Favorites(bool),
    Category(String),
    Copy(String),
    CopyPlain(String),
    PlainSelected,
    Copied(Result<(), String>),
    FinishCopy,
    Pin(String),
    Delete(String),
    Detail(Option<String>),
    EditCategory(String),
    SaveCategory,
    PauseMenu(bool),
    /// `Some(minutes)` pour une pause temporaire, `None` pour une pause sans limite.
    PauseFor(Option<u64>),
    Resume,
    Settings(bool),
    Density,
    Retention,
    ApplyRetention,
    KeepOpen,
    Language,
    Undo,
    Viewport(f32),
    WindowResized(Id, f32),
    AskClear,
    CancelClear,
    Clear,
    Page(bool),
    Move(bool),
    MoveRow(bool),
    Choose(usize),
    Enter,
    Escape,
    FocusSearch,
}

impl App {
    pub(crate) fn expand_demo(&mut self) {
        if !self.demo {
            return;
        }
        if let Some(store) = &mut self.store {
            for (index, text) in [
                "Bonjour,\nVoici les notes du jour.",
                "file:///home/demo/Documents/notes.txt",
                "#73B9FF",
                "fn main() {\n    println!(\"Bonjour COSMIC\");\n}",
            ]
            .iter()
            .enumerate()
            {
                if let Ok(clip) = Clip::new(
                    if index == 1 {
                        "text/uri-list"
                    } else {
                        "text/plain;charset=utf-8"
                    }
                    .into(),
                    text.as_bytes().to_vec(),
                    model::now() - 600 - index as i64,
                ) {
                    let _ = store.insert(&clip);
                }
            }
        }
        self.refresh();
    }
    fn filtered(&self) -> Vec<&Clip> {
        self.clips
            .iter()
            .filter(|clip| clip.matches(&self.query, self.kind, self.favorites, &self.category))
            .collect()
    }
    /// Largeur idéale du volet pour la densité courante ; le compositeur peut
    /// en accorder moins, ce que `viewport` rapporte.
    fn ideal_width(&self) -> f32 {
        match self.settings.density {
            Density::Compact => WIDTH_COMPACT,
            Density::Comfortable => WIDTH_WIDE,
        }
    }
    fn width(&self) -> f32 {
        self.viewport.clamp(WIDTH_MIN, self.ideal_width())
    }
    fn columns(&self) -> usize {
        match self.settings.density {
            Density::Compact => 1,
            Density::Comfortable => {
                let usable = self.width() - PADDING + CARD_SPACING;
                ((usable / (CARD_WIDTH + CARD_SPACING)) as usize).clamp(1, MAX_COLUMNS)
            }
        }
    }
    fn rows(&self) -> usize {
        match self.settings.density {
            Density::Compact => 6,
            Density::Comfortable => 2,
        }
    }
    /// Cartes affichées par page, donc aussi bornes de la sélection clavier.
    fn page_size(&self) -> usize {
        (self.columns() * self.rows()).max(1)
    }
    /// Déplace la sélection dans la page, quelle que soit la densité.
    fn step(&mut self, delta: isize) {
        let count = self
            .filtered()
            .len()
            .saturating_sub(self.page * self.page_size())
            .min(self.page_size());
        let last = count.saturating_sub(1) as isize;
        self.selected = (self.selected as isize + delta).clamp(0, last.max(0)) as usize;
    }
    fn flash(&mut self, message: impl Into<String>) {
        self.flash = Some((
            message.into(),
            Instant::now() + Duration::from_secs(FLASH_SECONDS),
        ));
    }
    fn save_settings(&mut self) {
        let Some(path) = self.settings_path.clone() else {
            return;
        };
        if let Err(e) = self.settings.save(&path) {
            self.status = tr_format!(
                "Préférences non enregistrées : {e}",
                "Preferences could not be saved: {e}"
            );
        }
    }
    /// Identifiant visé par une action clavier : l’aperçu ouvert, sinon la carte sélectionnée.
    fn target(&self) -> Option<String> {
        if let Some(id) = self.detail.clone() {
            return Some(id);
        }
        self.filtered()
            .get(self.page * self.page_size() + self.selected)
            .map(|clip| clip.id.clone())
    }
    fn reset(&mut self) {
        self.page = 0;
        self.selected = 0;
        self.clear_confirm = false;
    }
    fn refresh(&mut self) {
        // La rétention s’applique avant le chargement : rien de périmé n’est décodé.
        if let Some(store) = &self.store
            && self.settings.retention_days > 0
        {
            let cutoff = model::now() - i64::from(self.settings.retention_days) * 86_400;
            if let Err(e) = store.purge_older_than(cutoff) {
                self.status = e;
            }
        }
        if let Some(store) = &self.store {
            match store.load_cached(&mut self.clips) {
                Ok(clips) => self.clips = clips,
                Err(e) => {
                    self.status = e;
                    return;
                }
            }
        }
        self.thumbnails
            .retain(|id, _| self.clips.iter().any(|c| &c.id == id));
        self.payloads
            .retain(|id, _| self.clips.iter().any(|c| &c.id == id));
        for clip in &self.clips {
            self.payloads
                .entry(clip.id.clone())
                .or_insert_with(|| crate::transfer::Payload::from_clip(clip));
        }
        // Decode each thumbnail only once, and keep original bytes for copying.
        for clip in &self.clips {
            if clip.kind == Kind::Image
                && !self.thumbnails.contains_key(&clip.id)
                && let Ok(img) = model::decode_image(&clip.bytes)
            {
                let rgba = img.thumbnail(300, 180).to_rgba8();
                self.thumbnails.insert(
                    clip.id.clone(),
                    iced::widget::image::Handle::from_rgba(
                        rgba.width(),
                        rgba.height(),
                        rgba.into_raw(),
                    ),
                );
            }
        }
        let page_size = self.page_size();
        self.page = self
            .page
            .min(self.filtered().len().saturating_sub(1) / page_size);
        self.selected = self
            .selected
            .min(self.filtered().len().saturating_sub(1).min(page_size - 1));
    }
    fn write(&mut self, operation: impl FnOnce(&Store) -> Result<(), String>) -> bool {
        let Some(store) = &self.store else {
            self.status = tr!("Historique indisponible", "History unavailable").into();
            return false;
        };
        if let Err(e) = operation(store) {
            self.status = e;
            return false;
        }
        self.refresh();
        true
    }
    fn copy(&mut self, id: &str) -> Task<cosmic::Action<Message>> {
        if self.dragging {
            return Task::none();
        }
        if self.demo {
            self.status = tr!("Démonstration : copie désactivée", "Demo: copying disabled").into();
            return Task::none();
        }
        if self.copying {
            return Task::none();
        }
        if let Some(clip) = self.clips.iter().find(|c| c.id == id).cloned() {
            self.copying = true;
            self.status = tr!("Copie…", "Copying…").into();
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || clipboard::copy(clip))
                        .await
                        .map_err(|e| e.to_string())
                        .and_then(|r| r)
                },
                |result| cosmic::Action::App(Message::Copied(result)),
            );
        }
        Task::none()
    }
    /// Panneau de préférences. Chaque bouton fait défiler des valeurs connues :
    /// aucune saisie libre, donc aucune valeur invalide à écrire sur disque.
    fn view_settings(&self) -> Element<'_, Message> {
        let row = |label: &'static str, value: String, message: Message| {
            widget::row([])
                .push(widget::text(label).size(12).class(skin::MUTED))
                .push(widget::Space::new().width(Length::Fill))
                .push(
                    widget::button::text(value)
                        .class(skin::button(true, 7.0, false))
                        .on_press(message),
                )
                .spacing(8)
                .align_y(iced::Alignment::Center)
        };
        let location = if self.demo {
            tr!(
                "Démonstration : préférences en mémoire",
                "Demo: in-memory preferences"
            )
        } else if self.settings_path.is_some() {
            tr!(
                "Préférences enregistrées automatiquement",
                "Preferences saved automatically"
            )
        } else {
            tr!("Préférences indisponibles", "Preferences unavailable")
        };
        widget::container(
            widget::column([])
                .push(widget::text(tr!("Préférences", "Preferences")).size(15).class(skin::TEXT))
                .push(row(tr!("Langue", "Language"), self.settings.ui_language.into(), Message::Language))
                .push(row(
                    tr!("Affichage", "Layout"),
                    self.settings.density.label().into(),
                    Message::Density,
                ))
                .push(row(
                    tr!("Langue de l’OCR", "OCR language"),
                    self.settings.ocr_label().into(),
                    Message::OcrLanguage,
                ))
                .push(row(
                    tr!("Durée de rétention", "Retention"),
                    if self.retention_draft == 0 { tr!("Sans limite d’âge", "No age limit").into() } else { tr_format!("{} jours", "{} days", self.retention_draft) },
                    Message::Retention,
                ))
                .push(row(
                    tr!("Mode de collage", "Paste mode"),
                    self.settings.paste_label().into(),
                    Message::PasteMode,
                ))
                .push(row(tr!("Après copie", "After copying"), if self.settings.keep_open { tr!("Rester ouvert", "Keep open") } else { tr!("Fermer", "Close") }.into(), Message::KeepOpen))
                .push(widget::button::text(tr!("Appliquer la rétention", "Apply retention"))
                    .class(skin::button(false, 7.0, false)).on_press(Message::ApplyRetention))
                .push(widget::text(tr!("Appliquer efface les anciennes copies hors favoris.", "Applying removes old unpinned clips.")).size(11).class(skin::MUTED))
                .push(widget::text(location).size(11).class(skin::MUTED))
                .push(
                    widget::text(
                        tr!("La rétention ne supprime jamais les favoris. Les préférences sont relues au démarrage.", "Retention never removes favorites. Preferences are restored at startup."),
                    )
                    .size(11)
                    .class(skin::MUTED),
                )
                .spacing(9),
        )
        .padding(12)
        .width(Length::Fill)
        .class(skin::surface(skin::PREVIEW, 10.0, skin::LINE))
        .into()
    }
    /// Copie la même information sans sa mise en forme, en `text/plain`.
    /// Le contenu d’origine reste intact dans l’historique.
    fn copy_plain(&mut self, id: &str) -> Task<cosmic::Action<Message>> {
        if self.dragging || self.copying {
            return Task::none();
        }
        if self.demo {
            self.status = tr!("Démonstration : copie désactivée", "Demo: copying disabled").into();
            return Task::none();
        }
        let plain = self
            .clips
            .iter()
            .find(|c| c.id == id)
            .map(model::plain_text);
        let text = match plain {
            Some(Some(text)) => text,
            Some(None) => {
                self.status = tr!(
                    "Texte brut indisponible pour une image ; utilise l’OCR.",
                    "Images cannot be copied as plain text; use OCR."
                )
                .into();
                return Task::none();
            }
            None => return Task::none(),
        };
        match Clip::new(
            "text/plain;charset=utf-8".into(),
            text.into_bytes(),
            model::now(),
        ) {
            Ok(plain) => {
                self.copying = true;
                self.status = tr!("Copie en texte brut…", "Copying as plain text…").into();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || clipboard::copy(plain))
                            .await
                            .map_err(|e| e.to_string())
                            .and_then(|r| r)
                    },
                    |result| cosmic::Action::App(Message::Copied(result)),
                )
            }
            Err(e) => {
                self.status = e;
                Task::none()
            }
        }
    }
    fn preview<'a>(&self, clip: &'a Clip, height: f32) -> Element<'a, Message> {
        let content: Element<'a, Message> = if let Some(handle) = self.thumbnails.get(&clip.id) {
            widget::image(handle.clone())
                .width(Length::Fill)
                .height(height)
                .content_fit(iced::ContentFit::Contain)
                .into()
        } else if let Some(rgb) = model::color(clip.text.trim()) {
            let light = u32::from(rgb[0]) * 299 + u32::from(rgb[1]) * 587 + u32::from(rgb[2]) * 114
                > 150000;
            widget::container(widget::text(clip.text.trim()).size(25).class(if light {
                Color::BLACK
            } else {
                Color::WHITE
            }))
            .center_x(Length::Fill)
            .center_y(height)
            .class(skin::surface(
                Color::from_rgb8(rgb[0], rgb[1], rgb[2]),
                8.0,
                Color::TRANSPARENT,
            ))
            .into()
        } else if clip.kind == Kind::Link && height < 180.0 {
            let domain = clip
                .text
                .trim()
                .split("://")
                .nth(1)
                .unwrap_or(&clip.text)
                .split('/')
                .next()
                .unwrap_or(tr!("Lien web", "Web link"));
            widget::container(
                widget::column([])
                    .push(skin::icon("insert-link-symbolic").icon().size(24))
                    .push(
                        widget::text(domain.chars().take(23).collect::<String>())
                            .size(17)
                            .class(skin::kind_color(Kind::Link)),
                    )
                    .push(
                        widget::text(tr!("Lien enregistré", "Saved link"))
                            .size(11)
                            .class(skin::MUTED),
                    )
                    .spacing(8),
            )
            .padding(14)
            .width(Length::Fill)
            .height(height)
            .into()
        } else if clip.kind == Kind::Files && height < 180.0 {
            let count = clip
                .text
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .count();
            widget::container(
                widget::column([])
                    .push(skin::icon("folder-symbolic").icon().size(28))
                    .push(
                        widget::text(tr_format!("{count} fichier(s)", "{count} file(s)"))
                            .size(18)
                            .class(skin::kind_color(Kind::Files)),
                    )
                    .push(
                        widget::text(tr!("Emplacements d’origine", "Original locations"))
                            .size(11)
                            .class(skin::MUTED),
                    )
                    .spacing(8),
            )
            .padding(14)
            .width(Length::Fill)
            .height(height)
            .into()
        } else {
            let text = clip
                .text
                .lines()
                .take(if height > 180.0 { 12 } else { 4 })
                .collect::<Vec<_>>()
                .join("\n")
                .chars()
                .take(500)
                .collect::<String>();
            let text = widget::text(text).size(13).class(skin::TEXT);
            let text = if clip.kind == Kind::Code {
                text.font(cosmic::font::mono())
            } else {
                text
            };
            widget::container(text)
                .width(Length::Fill)
                .height(height)
                .padding(12)
                .into()
        };
        widget::container(content)
            .clip(true)
            .width(Length::Fill)
            .height(height)
            .class(skin::surface(skin::PREVIEW, 8.0, Color::TRANSPARENT))
            .into()
    }
    /// Poignée de glisser-déposer, identique dans les deux densités.
    fn drag_source<'a>(&self, clip: &'a Clip) -> Element<'a, Message> {
        let payload = self
            .payloads
            .get(&clip.id)
            .expect("refreshed payload")
            .clone();
        let drag = widget::DndSource::with_id(
            widget::container(widget::text("⠿").size(20).class(skin::MUTED)).padding([3, 9]),
            iced::widget::Id::new(format!("drag-{}", clip.id)),
        )
        .action(iced::clipboard::dnd::DndAction::Copy)
        .on_start(Some(Message::Drag(true)))
        .on_finish(Some(Message::Drag(false)))
        .on_cancel(Some(Message::Drag(false)));
        if self.demo {
            drag.into()
        } else {
            drag.drag_content(move || payload.clone()).into()
        }
    }
    /// Mêmes actions dans les deux densités : rien n’est masqué en mode compact,
    /// seuls les intitulés deviennent des icônes.
    fn actions<'a>(&self, clip: &'a Clip, compact: bool) -> Element<'a, Message> {
        let preview: Element<'a, Message> = if compact {
            widget::button::icon(skin::icon("view-reveal-symbolic"))
                .class(skin::button(false, 6.0, false))
                .on_press(Message::Detail(Some(clip.id.clone())))
                .into()
        } else {
            widget::button::text(tr!("Aperçu", "Preview"))
                .class(skin::button(false, 6.0, false))
                .on_press(Message::Detail(Some(clip.id.clone())))
                .into()
        };
        let mut row = widget::row([])
            .push(self.drag_source(clip))
            .push(
                widget::button::text(if clip.pinned { "★" } else { "☆" })
                    .class(skin::button(clip.pinned, 6.0, false))
                    .on_press(Message::Pin(clip.id.clone())),
            )
            .push(preview);
        if !compact {
            row = row.push(widget::Space::new().width(Length::Fill));
        }
        row.push(
            widget::button::icon(skin::icon("edit-delete-symbolic"))
                .class(skin::button(false, 6.0, false))
                .on_press(Message::Delete(clip.id.clone())),
        )
        .spacing(2)
        .align_y(iced::Alignment::Center)
        .into()
    }
    fn card<'a>(&self, clip: &'a Clip, index: usize) -> Element<'a, Message> {
        match self.settings.density {
            Density::Compact => self.card_list(clip, index),
            Density::Comfortable => self.card_grid(clip, index),
        }
    }
    /// Ligne compacte : une colonne, aperçu réduit, actions à droite.
    fn card_list<'a>(&self, clip: &'a Clip, index: usize) -> Element<'a, Message> {
        let summary = widget::column([])
            .push(
                widget::row([])
                    .push(skin::icon(clip.kind.icon()).icon().size(13))
                    .push(
                        widget::text(clip.kind.label())
                            .size(11)
                            .class(skin::kind_color(clip.kind)),
                    )
                    .push(widget::Space::new().width(Length::Fill))
                    .push(
                        widget::text(if index < MAX_SHORTCUTS {
                            format!("⌃{}", index + 1)
                        } else {
                            String::new()
                        })
                        .size(11)
                        .class(skin::MUTED),
                    )
                    .spacing(6)
                    .align_y(iced::Alignment::Center),
            )
            .push(
                widget::text(clip.title.chars().take(52).collect::<String>())
                    .size(13)
                    .class(skin::TEXT),
            )
            .push(
                widget::text(format!(
                    "{}{}",
                    model::age(clip.timestamp),
                    if clip.category.is_empty() {
                        String::new()
                    } else {
                        format!(" · {}", clip.category.chars().take(16).collect::<String>())
                    }
                ))
                .size(11)
                .class(skin::MUTED),
            )
            .spacing(4)
            .width(Length::Fill);
        let copy = widget::button::custom(
            widget::row([])
                .push(
                    widget::container(self.preview(clip, 54.0))
                        .width(76)
                        .clip(true),
                )
                .push(summary)
                .spacing(10)
                .align_y(iced::Alignment::Center),
        )
        .on_press(Message::Copy(clip.id.clone()))
        .padding(10)
        .width(Length::Fill)
        .class(skin::button(self.selected == index, 10.0, true));
        widget::row([])
            .push(copy)
            .push(self.actions(clip, true))
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .into()
    }
    fn card_grid<'a>(&self, clip: &'a Clip, index: usize) -> Element<'a, Message> {
        let header = widget::row([])
            .push(skin::icon(clip.kind.icon()).icon().size(14))
            .push(
                widget::text(clip.kind.label())
                    .size(11)
                    .class(skin::kind_color(clip.kind)),
            )
            .push(widget::Space::new().width(Length::Fill))
            .push(
                widget::text(if index < MAX_SHORTCUTS {
                    format!("⌃{}", index + 1)
                } else {
                    String::new()
                })
                .size(11)
                .class(skin::MUTED),
            )
            .spacing(6);
        let body = widget::column([])
            .push(
                widget::container(widget::Space::new().height(3))
                    .width(Length::Fill)
                    .class(skin::surface(
                        skin::kind_color(clip.kind),
                        2.0,
                        Color::TRANSPARENT,
                    )),
            )
            .push(header)
            .push(self.preview(clip, 124.0))
            .push(
                widget::text(clip.title.chars().take(26).collect::<String>())
                    .size(13)
                    .class(skin::TEXT),
            )
            .push(
                widget::text(format!(
                    "{}{}",
                    model::age(clip.timestamp),
                    if clip.category.is_empty() {
                        String::new()
                    } else {
                        format!(" · {}", clip.category.chars().take(16).collect::<String>())
                    }
                ))
                .size(11)
                .class(skin::MUTED),
            )
            .spacing(9);
        let copy = widget::button::custom(body)
            .on_press(Message::Copy(clip.id.clone()))
            .padding(12)
            .width(Length::Fill)
            .class(skin::button(self.selected == index, 12.0, true));
        widget::column([])
            .push(copy)
            .push(self.actions(clip, false))
            .spacing(3)
            .width(Length::FillPortion(1))
            .into()
    }
}

impl cosmic::Application for App {
    type Executor = cosmic::executor::Default;
    type Flags = Mode;
    type Message = Message;
    const APP_ID: &'static str = "io.github.nebulapaste.NebulaPaste";
    fn core(&self) -> &cosmic::Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }
    fn init(mut core: cosmic::Core, mode: Mode) -> (Self, Task<cosmic::Action<Message>>) {
        let (demo, instance) = match mode {
            Mode::Applet(i) => (false, Some(i)),
            Mode::Preview => (true, None),
        };
        if demo {
            core.window.show_headerbar = false;
            core.window.content_container = false;
        }
        let result = if demo {
            Store::in_memory()
        } else {
            dirs::data_local_dir()
                .ok_or(
                    tr!(
                        "Dossier utilisateur introuvable",
                        "User directory not found"
                    )
                    .into(),
                )
                .and_then(|p| Store::open(&p.join("nebula-paste/history.sqlite3")))
        };
        let (store, error) = match result {
            Ok(s) => (Some(s), None),
            Err(e) => (None, Some(e)),
        };
        // La démonstration n’écrit ni ne lit les préférences de l’utilisateur.
        let settings_path = if demo { None } else { Settings::default_path() };
        let settings = settings_path
            .as_deref()
            .map(Settings::load)
            .unwrap_or_default();
        if !cfg!(test) {
            nebula_paste::i18n::set_language(settings.ui_language);
        }
        let mut app = Self {
            core,
            popup: None,
            instance,
            demo,
            monitor: if demo {
                Monitor::idle()
            } else {
                Monitor::start(error.is_some())
            },
            store,
            clips: vec![],
            thumbnails: HashMap::new(),
            payloads: HashMap::new(),
            query: String::new(),
            kind: None,
            favorites: false,
            category: String::new(),
            category_edit: String::new(),
            detail: None,
            page: 0,
            selected: 0,
            status: error.unwrap_or(tr!("Connexion à Wayland…", "Connecting to Wayland…").into()),
            connected: false,
            clear_confirm: false,
            copying: false,
            search_id: iced::widget::Id::unique(),
            ocr_busy: false,
            dragging: false,
            retention_draft: settings.retention_days,
            settings,
            settings_path,
            settings_open: false,
            pause_menu: false,
            pause_until: None,
            viewport: WIDTH_WIDE,
            undo: None,
            flash: None,
        };
        if demo && let Some(store) = &mut app.store {
            for clip in crate::demo::clips() {
                let _ = store.insert(&clip);
                let _ = store.pin(&clip.id, clip.pinned);
                let _ = store.category(&clip.id, &clip.category);
            }
        }
        app.refresh();
        (app, Task::none())
    }
    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::Closed(id))
    }
    fn view(&self) -> Element<'_, Message> {
        if self.demo {
            return self.view_window(Id::unique());
        }
        self.core
            .applet
            .icon_button(if self.monitor.paused() {
                "media-playback-pause-symbolic"
            } else {
                "io.github.nebulapaste.NebulaPaste-symbolic"
            })
            .on_press(Message::Toggle)
            .into()
    }
    fn view_window(&self, _: Id) -> Element<'_, Message> {
        let compact = self.settings.density == Density::Compact || self.width() < 600.0;
        let paused = self.monitor.paused() && !self.demo;
        let mut identity = widget::column([]).push(
            widget::text("Nebula Paste")
                .size(if compact { 17 } else { 23 })
                .class(skin::TEXT),
        );
        if !compact {
            identity = identity.push(
                widget::text(tr!("Copiez. Retrouvez. Créez.", "Copy. Find. Create."))
                    .size(12)
                    .class(skin::MUTED),
            );
        }
        let paste_button = if compact {
            Element::from(
                widget::button::icon(skin::icon("edit-paste-symbolic"))
                    .class(skin::button(self.settings.paste_mode != 0, 8.0, false))
                    .on_press(Message::PasteMode),
            )
        } else {
            Element::from(
                widget::button::text(self.settings.paste_label())
                    .class(skin::button(self.settings.paste_mode != 0, 8.0, false))
                    .on_press(Message::PasteMode),
            )
        };
        let pause_button: Element<'_, Message> = if compact {
            widget::button::icon(skin::icon(if paused {
                "media-playback-start-symbolic"
            } else {
                "media-playback-pause-symbolic"
            }))
            .class(skin::button(paused || self.pause_menu, 8.0, false))
            .on_press(if paused {
                Message::Resume
            } else {
                Message::PauseMenu(!self.pause_menu)
            })
            .into()
        } else if paused {
            widget::button::text(tr!("Reprendre", "Resume"))
                .class(skin::button(true, 8.0, false))
                .on_press(Message::Resume)
                .into()
        } else {
            widget::button::text(tr!("Pause", "Pause"))
                .class(skin::button(self.pause_menu, 8.0, false))
                .on_press(Message::PauseMenu(!self.pause_menu))
                .into()
        };
        let title = widget::row([])
            .push(
                skin::brand_icon()
                    .icon()
                    .size(if compact { 24 } else { 40 }),
            )
            .push(identity.spacing(3))
            .push(widget::Space::new().width(Length::Fill))
            .push(paste_button)
            .push(pause_button)
            .push(
                widget::button::icon(skin::icon("emblem-system-symbolic"))
                    .class(skin::button(self.settings_open, 8.0, false))
                    .on_press(Message::Settings(!self.settings_open)),
            )
            .push(
                widget::button::icon(skin::icon("window-close-symbolic"))
                    .class(skin::button(false, 8.0, false))
                    .on_press(Message::Toggle),
            )
            .spacing(if compact { 4 } else { 10 })
            .align_y(iced::Alignment::Center);
        let mut layout = widget::column([]).push(title).spacing(14);
        if paused {
            layout = layout.push(
                widget::container(
                    widget::row([])
                        .push(skin::icon("media-playback-pause-symbolic").icon().size(16))
                        .push(
                            widget::text(match self.pause_until {
                                Some(end) => tr_format!(
                                    "Capture en pause · reprise dans {}",
                                    "Capture paused · resumes in {}",
                                    model::countdown(
                                        end.saturating_duration_since(Instant::now()).as_secs()
                                            as i64
                                    )
                                ),
                                None => tr!(
                                    "Capture en pause · sans limite de durée",
                                    "Capture paused until resumed"
                                )
                                .into(),
                            })
                            .size(12)
                            .class(skin::TEXT),
                        )
                        .push(widget::Space::new().width(Length::Fill))
                        .push(
                            widget::button::text(tr!("Reprendre maintenant", "Resume now"))
                                .class(skin::button(true, 7.0, false))
                                .on_press(Message::Resume),
                        )
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                )
                .padding([6, 10])
                .width(Length::Fill)
                .class(skin::surface(skin::PREVIEW, 8.0, skin::LINE)),
            );
        } else if self.pause_menu {
            let mut durations = widget::row([])
                .push(
                    widget::text(tr!("Suspendre la capture", "Pause capture"))
                        .size(12)
                        .class(skin::MUTED),
                )
                .spacing(6)
                .align_y(iced::Alignment::Center);
            for minutes in PAUSES {
                durations = durations.push(
                    widget::button::text(format!("{minutes} min"))
                        .class(skin::button(false, 7.0, false))
                        .on_press(Message::PauseFor(Some(minutes))),
                );
            }
            layout = layout.push(
                widget::container(
                    durations
                        .push(
                            widget::button::text(tr!("Sans limite", "Until resumed"))
                                .class(skin::button(false, 7.0, false))
                                .on_press(Message::PauseFor(None)),
                        )
                        .push(widget::Space::new().width(Length::Fill))
                        .push(
                            widget::button::text(tr!("Annuler", "Cancel"))
                                .class(skin::button(false, 7.0, false))
                                .on_press(Message::PauseMenu(false)),
                        ),
                )
                .padding([6, 10])
                .width(Length::Fill)
                .class(skin::surface(skin::PREVIEW, 8.0, skin::LINE)),
            );
        }
        if self.settings_open {
            layout = layout.push(self.view_settings());
        }
        if !self.settings_open {
            if let Some(clip) = self
                .detail
                .as_ref()
                .and_then(|id| self.clips.iter().find(|c| &c.id == id))
            {
                let body: Element<'_, Message> =
                    if clip.kind == Kind::Image || clip.kind == Kind::Color {
                        self.preview(clip, 250.0)
                    } else {
                        widget::scrollable(
                            widget::text(clip.text.chars().take(20000).collect::<String>())
                                .size(14)
                                .class(skin::TEXT),
                        )
                        .height(250)
                        .into()
                    };
                layout = layout
                    .push(
                        widget::button::text(tr!("← Historique", "← History"))
                            .class(skin::button(false, 8.0, false))
                            .on_press(Message::Detail(None)),
                    )
                    .push(widget::text(&clip.title).size(18).class(skin::TEXT))
                    .push(body)
                    .push(
                        widget::text(tr_format!(
                            "{} · {} octets · {}",
                            "{} · {} bytes · {}",
                            clip.kind.label(),
                            clip.bytes.len(),
                            model::age(clip.timestamp)
                        ))
                        .size(12)
                        .class(skin::MUTED),
                    )
                    .push(
                        widget::row([])
                            .push(
                                widget::text_input(
                                    tr!(
                                        "Catégorie : Travail, Commandes…",
                                        "Collection: Work, Commands…"
                                    ),
                                    &self.category_edit,
                                )
                                .style(skin::input())
                                .padding(10)
                                .on_input(Message::EditCategory)
                                .on_submit(|_| Message::SaveCategory),
                            )
                            .push(
                                widget::button::standard(tr!("Enregistrer", "Save"))
                                    .class(skin::button(true, 8.0, false))
                                    .on_press(Message::SaveCategory),
                            )
                            .spacing(8),
                    )
                    .push(
                        widget::row([])
                            .push(
                                widget::button::suggested(if self.settings.paste_mode == 0 {
                                    tr!("Copier le contenu", "Copy content")
                                } else {
                                    tr!("Copier et coller", "Copy and paste")
                                })
                                .class(skin::button(true, 8.0, true))
                                .on_press(Message::Copy(clip.id.clone())),
                            )
                            .push(
                                widget::button::text(tr!(
                                    "Copier en texte brut · Ctrl+Maj+C",
                                    "Copy as plain text · Ctrl+Shift+C"
                                ))
                                .class(skin::button(false, 8.0, false))
                                .on_press_maybe(
                                    model::plain_text(clip)
                                        .is_some()
                                        .then(|| Message::CopyPlain(clip.id.clone())),
                                ),
                            )
                            .spacing(8)
                            .align_y(iced::Alignment::Center),
                    );
                if clip.kind == Kind::Image {
                    layout = layout.push(
                        widget::row([])
                            .push(
                                widget::button::text(if self.ocr_busy {
                                    tr!("Lecture de l’image…", "Reading image…")
                                } else {
                                    tr!("Extraire le texte · OCR", "Extract text · OCR")
                                })
                                .class(skin::button(true, 8.0, true))
                                .on_press_maybe(
                                    (!self.ocr_busy).then(|| Message::Ocr(clip.id.clone())),
                                ),
                            )
                            .push(
                                widget::button::text(self.settings.ocr_label())
                                    .class(skin::button(false, 8.0, false))
                                    .on_press(Message::OcrLanguage),
                            )
                            .push(widget::Space::new().width(Length::Fill))
                            .push(
                                widget::text(tr!(
                                    "OCR intégré · hors ligne",
                                    "Built-in OCR · offline"
                                ))
                                .size(12)
                                .class(skin::MUTED),
                            )
                            .align_y(iced::Alignment::Center)
                            .spacing(8),
                    );
                }
            } else {
                let mut tabs = widget::row([])
                    .spacing(6)
                    .push(
                        widget::button::text(tr!("Historique", "History"))
                            .class(skin::button(
                                !self.favorites && self.category.is_empty(),
                                8.0,
                                false,
                            ))
                            .on_press(Message::Favorites(false)),
                    )
                    .push(
                        widget::button::text(tr!("★ Favoris", "★ Favorites"))
                            .class(skin::button(self.favorites, 8.0, false))
                            .on_press(Message::Favorites(true)),
                    );
                let mut categories: Vec<_> = self
                    .clips
                    .iter()
                    .map(|c| c.category.as_str())
                    .filter(|c| !c.is_empty())
                    .collect();
                categories.sort();
                categories.dedup();
                for category in categories {
                    tabs = tabs.push(
                        widget::button::text(category)
                            .class(skin::button(self.category == category, 8.0, false))
                            .on_press(Message::Category(category.into())),
                    );
                }
                let tabs = widget::scrollable(tabs)
                    .direction(iced::widget::scrollable::Direction::Horizontal(
                        iced::widget::scrollable::Scrollbar::default(),
                    ))
                    .width(Length::Fill);
                layout = layout
                    .push(
                        widget::row([])
                            .push(
                                widget::search_input(
                                    tr!("Rechercher une copie…", "Search clipboard history…"),
                                    &self.query,
                                )
                                .leading_icon(skin::icon("search").icon().size(16).into())
                                .style(skin::input())
                                .padding(10)
                                .width(Length::Fill)
                                .id(self.search_id.clone())
                                .on_input(Message::Search)
                                .on_submit(|_| Message::Enter),
                            )
                            .spacing(18)
                            .align_y(iced::Alignment::Center),
                    )
                    .push(tabs);
                let mut filters = widget::row([]).spacing(4);
                for kind in std::iter::once(None).chain(Kind::ALL.into_iter().map(Some)) {
                    filters = filters.push(
                        widget::button::text(kind.map_or(tr!("Tout", "All"), Kind::label))
                            .class(skin::button(self.kind == kind, 7.0, false))
                            .on_press(Message::Filter(kind)),
                    );
                }
                let filtered = self.filtered();
                filters = filters.push(widget::Space::new().width(Length::Fill)).push(
                    widget::text(tr_format!("{} copies", "{} clips", filtered.len()))
                        .size(12)
                        .class(skin::MUTED),
                );
                layout = layout.push(
                    widget::scrollable(filters.align_y(iced::Alignment::Center)).direction(
                        iced::widget::scrollable::Direction::Horizontal(
                            iced::widget::scrollable::Scrollbar::default(),
                        ),
                    ),
                );
                let page_size = self.page_size();
                let columns = self.columns();
                let page: Vec<_> = filtered
                    .iter()
                    .skip(self.page * page_size)
                    .take(page_size)
                    .collect();
                let mut grid = widget::column([]).spacing(if compact { 8 } else { 14 });
                if page.is_empty() {
                    grid=grid.push(widget::container(widget::column([])
                .push(widget::text(if self.query.is_empty() {tr!("Tout commence par une copie.", "It starts with a copy.")} else {tr!("Aucun résultat.", "No results.")}).size(22).class(skin::TEXT))
                .push(widget::text(tr!("Textes, images, liens… retrouve-les ici, quand tu en as besoin.", "Text, images, links… find them here whenever you need them.")).size(13).class(skin::MUTED)).spacing(12)).center_x(Length::Fill).center_y(200));
                }
                for (row_index, chunk) in page.chunks(columns).enumerate() {
                    let mut row = widget::row([]).spacing(12);
                    for (col, clip) in chunk.iter().enumerate() {
                        row = row.push(self.card(clip, row_index * columns + col));
                    }
                    for _ in chunk.len()..columns {
                        row = row.push(widget::Space::new().width(Length::FillPortion(1)));
                    }
                    grid = grid.push(row);
                }
                let height = if compact {
                    420.0
                } else if page.len() > columns {
                    480.0
                } else {
                    280.0
                };
                layout = layout.push(widget::scrollable(grid).height(height));
                let page_count = filtered.len().max(1).div_ceil(page_size);
                layout = layout.push(
                    widget::column([])
                        .push(
                            widget::text(tr_format!(
                                "⠿ Glisser · Ctrl+1…{} : choisir · Ctrl+Maj+C : texte brut",
                                "⠿ Drag · Ctrl+1…{}: select · Ctrl+Shift+C: plain text",
                                page_size.min(MAX_SHORTCUTS)
                            ))
                            .size(11)
                            .class(skin::MUTED),
                        )
                        .push(
                            widget::row([])
                                .push(
                                    widget::button::text(tr!("Vider…", "Clear…"))
                                        .class(skin::button(false, 7.0, false))
                                        .on_press(Message::AskClear),
                                )
                                .push(widget::Space::new().width(Length::Fill))
                                .push(
                                    widget::button::text("←")
                                        .class(skin::button(false, 7.0, false))
                                        .on_press_maybe(
                                            (self.page > 0).then_some(Message::Page(false)),
                                        ),
                                )
                                .push(
                                    widget::text(format!("{} / {page_count}", self.page + 1))
                                        .size(12)
                                        .class(skin::MUTED),
                                )
                                .push(
                                    widget::button::text("→")
                                        .class(skin::button(false, 7.0, false))
                                        .on_press_maybe(
                                            (self.page + 1 < page_count)
                                                .then_some(Message::Page(true)),
                                        ),
                                )
                                .spacing(8)
                                .align_y(iced::Alignment::Center),
                        )
                        .spacing(8),
                );
            }
        }
        let message = self
            .flash
            .as_ref()
            .map(|(text, _)| text.as_str())
            .unwrap_or_else(|| {
                if self.demo
                    && (self.status.starts_with("Connexion")
                        || self.status.starts_with("Connecting"))
                {
                    tr!(
                        "Démonstration · historique isolé",
                        "Demo · isolated history"
                    )
                } else {
                    &self.status
                }
            });
        let mut state = widget::column([]).spacing(8).push(
            widget::text(message)
                .width(Length::Fill)
                .size(12)
                .class(if self.flash.is_some() {
                    skin::ACCENT
                } else {
                    skin::MUTED
                }),
        );
        if let Some(undo) = &self.undo {
            let remaining = undo
                .until
                .saturating_duration_since(Instant::now())
                .as_secs();
            state = state.push(
                widget::row([])
                    .push(
                        widget::text(tr_format!(
                            "Suppression · {remaining} s",
                            "Deleted · {remaining} s"
                        ))
                        .size(11)
                        .class(skin::MUTED),
                    )
                    .push(widget::Space::new().width(Length::Fill))
                    .push(
                        widget::button::text(tr!("Annuler la suppression", "Undo deletion"))
                            .class(skin::button(true, 7.0, false))
                            .on_press(Message::Undo),
                    )
                    .align_y(iced::Alignment::Center)
                    .spacing(8),
            );
        }
        layout = layout.push(
            widget::container(state)
                .padding([8, 10])
                .width(Length::Fill)
                .class(skin::surface(skin::PREVIEW, 8.0, skin::LINE)),
        );
        if self.clear_confirm {
            layout = layout.push(
                widget::row([])
                    .push(
                        widget::text(tr!(
                            "Effacer l’historique hors favoris ?",
                            "Clear history except favorites?"
                        ))
                        .class(skin::TEXT),
                    )
                    .push(
                        widget::button::text(tr!("Annuler", "Cancel"))
                            .class(skin::button(false, 8.0, false))
                            .on_press(Message::CancelClear),
                    )
                    .push(
                        widget::button::text(tr!("Effacer", "Clear"))
                            .class(skin::button(true, 8.0, false))
                            .on_press(Message::Clear),
                    )
                    .spacing(8),
            );
        }
        let content = widget::container(layout)
            .width(self.width())
            .padding(if compact { 12 } else { 18 })
            .class(skin::surface(skin::BG, 16.0, skin::LINE));
        if self.demo {
            content.into()
        } else {
            self.core
                .applet
                .popup_container(content)
                .limits(
                    Limits::NONE
                        .min_width(WIDTH_MIN)
                        .max_width(WIDTH_WIDE)
                        .max_height(850.0),
                )
                .into()
        }
    }
    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions =
            vec![iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)];
        if self.popup.is_some() || self.demo {
            subscriptions.push(iced::event::listen_with(|event, status, id| {
                // La largeur réellement accordée par le compositeur pilote la mise en page.
                if let iced::Event::Window(iced::window::Event::Resized(size)) = event {
                    return Some(Message::WindowResized(id, size.width));
                }
                let iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key,
                    modifiers: mods,
                    ..
                }) = event
                else {
                    return None;
                };
                match key.as_ref() {
                    Key::Named(Named::Escape) => Some(Message::Escape),
                    Key::Character(c)
                        if mods.control() && mods.shift() && c.eq_ignore_ascii_case("c") =>
                    {
                        Some(Message::PlainSelected)
                    }
                    Key::Character("f") if mods.control() => Some(Message::FocusSearch),
                    Key::Named(Named::ArrowRight) if mods.alt() => Some(Message::Move(true)),
                    Key::Named(Named::ArrowLeft) if mods.alt() => Some(Message::Move(false)),
                    Key::Named(Named::ArrowDown) if mods.alt() => Some(Message::MoveRow(true)),
                    Key::Named(Named::ArrowUp) if mods.alt() => Some(Message::MoveRow(false)),
                    Key::Character(c) if mods.control() => c
                        .parse::<usize>()
                        .ok()
                        .filter(|n| (1..=MAX_SHORTCUTS).contains(n))
                        .map(|n| Message::Choose(n - 1)),
                    Key::Named(Named::Enter) if status == iced::event::Status::Ignored => {
                        Some(Message::Enter)
                    }
                    _ => None,
                }
            }));
        }
        Subscription::batch(subscriptions)
    }
    fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::FocusSearch => {
                self.detail = None;
                return widget::text_input::focus(self.search_id.clone());
            }
            Message::Escape => {
                if self.clear_confirm {
                    self.clear_confirm = false;
                } else if self.pause_menu {
                    self.pause_menu = false;
                } else if self.settings_open {
                    self.settings_open = false;
                } else if self.detail.is_some() {
                    self.detail = None;
                } else {
                    return self.update(Message::Toggle);
                }
            }
            Message::Drag(value) => {
                self.dragging = value;
            }
            Message::PasteMode => {
                self.settings.paste_mode = (self.settings.paste_mode + 1) % 3;
                self.status = match self.settings.paste_mode {
                    1 => tr!("Collage direct · Ctrl+V envoyé à l’application active après fermeture. Nécessite wtype.", "Direct paste · sends Ctrl+V to the active application after closing. Requires wtype."),
                    2 => tr!("Mode terminal · Ctrl+Maj+V envoyé après fermeture. Nécessite wtype.", "Terminal mode · sends Ctrl+Shift+V after closing. Requires wtype."),
                    _ => tr!("Copie seule · colle ensuite avec Ctrl+V.", "Copy only · paste afterwards using Ctrl+V."),
                }.into();
                self.save_settings();
            }
            Message::OcrLanguage => {
                self.settings.cycle_ocr_language();
                self.save_settings();
            }
            Message::Density => {
                self.settings.density = self.settings.density.next();
                self.viewport = self.ideal_width();
                self.reset();
                self.refresh();
                self.flash(tr_format!(
                    "Affichage : {}",
                    "Layout: {}",
                    self.settings.density.label()
                ));
                self.save_settings();
            }
            Message::Language => {
                self.settings.ui_language = match self.settings.ui_language {
                    "auto" => "en",
                    "en" => "fr",
                    _ => "auto",
                };
                nebula_paste::i18n::set_language(self.settings.ui_language);
                self.save_settings();
            }
            Message::KeepOpen => {
                self.settings.keep_open = !self.settings.keep_open;
                self.save_settings();
            }
            Message::Retention => {
                let values = nebula_paste::settings::RETENTIONS;
                let index = values
                    .iter()
                    .position(|&v| v == self.retention_draft)
                    .unwrap_or(0);
                self.retention_draft = values[(index + 1) % values.len()];
            }
            Message::ApplyRetention => {
                self.settings.retention_days = self.retention_draft;
                self.save_settings();
                self.undo = None;
                self.refresh();
                self.flash(tr_format!(
                    "Rétention : {}",
                    "Retention: {}",
                    self.settings.retention_label()
                ));
            }
            Message::Settings(open) => {
                self.settings_open = open;
                self.pause_menu = false;
            }
            Message::WindowResized(id, width) => {
                if self.popup == Some(id) || self.demo {
                    return self.update(Message::Viewport(width));
                }
            }
            Message::Viewport(width) => {
                if width.is_finite() && (self.viewport - width).abs() > 1.0 {
                    self.viewport = width;
                    self.refresh();
                }
            }
            Message::Ocr(id) => {
                if self.ocr_busy {
                    return Task::none();
                }
                if let Some(clip) = self
                    .clips
                    .iter()
                    .find(|c| c.id == id && c.kind == Kind::Image)
                    .cloned()
                {
                    self.ocr_busy = true;
                    self.status = tr!(
                        "Reconnaissance locale en cours…",
                        "Recognizing text locally…"
                    )
                    .into();
                    let language = self.settings.ocr_language;
                    return Task::perform(
                        async move {
                            let result = tokio::task::spawn_blocking(move || {
                                crate::actions::ocr(clip, language)
                            })
                            .await
                            .map_err(|e| e.to_string())
                            .and_then(|r| r);
                            (id, result)
                        },
                        |(id, result)| cosmic::Action::App(Message::OcrDone(id, result)),
                    );
                }
            }
            Message::OcrDone(source, result) => {
                self.ocr_busy = false;
                // Never resurrect content whose source was deleted during recognition.
                if !self.clips.iter().any(|c| c.id == source) {
                    return Task::none();
                }
                let result = result.and_then(|text| {
                    Clip::new(
                        "text/plain;charset=utf-8".into(),
                        text.into_bytes(),
                        model::now(),
                    )
                });
                match result {
                    Ok(clip) => {
                        if let Some(store) = &mut self.store {
                            match store.insert(&clip) {
                                Ok(()) => {
                                    self.refresh();
                                    self.status = tr!(
                                        "Texte extrait · vérifie le résultat avant de le copier.",
                                        "Text extracted · review it before copying."
                                    )
                                    .into();
                                    return self.update(Message::Detail(Some(clip.id)));
                                }
                                Err(e) => self.status = e,
                            }
                        }
                    }
                    Err(e) => self.status = e,
                }
            }
            Message::PasteReady(terminal) => {
                if self.popup.is_some() || self.demo {
                    self.copying = false;
                    return Task::none();
                }
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || crate::actions::paste(terminal))
                            .await
                            .map_err(|e| e.to_string())
                            .and_then(|r| r)
                    },
                    |result| cosmic::Action::App(Message::Pasted(result)),
                );
            }
            Message::Pasted(result) => {
                self.copying = false;
                match result {
                    Ok(()) => {
                        self.status =
                            tr!("Raccourci de collage envoyé.", "Paste shortcut sent.").into()
                    }
                    Err(e) => {
                        self.status = e;
                        if self.popup.is_none() && !self.demo {
                            return self.update(Message::Toggle);
                        }
                    }
                }
            }
            Message::Toggle => {
                if self.demo {
                    return iced::exit();
                }
                if let Some(id) = self.popup.take() {
                    self.dragging = false;
                    return destroy_popup(id);
                }
                let Some(parent) = self.core.main_window_id() else {
                    return Task::none();
                };
                let id = Id::unique();
                self.popup = Some(id);
                self.detail = None;
                self.dragging = false;
                self.clear_confirm = false;
                self.settings_open = false;
                self.pause_menu = false;
                let mut settings = self
                    .core
                    .applet
                    .get_popup_settings(parent, id, None, None, None);
                settings.positioner.size_limits = Limits::NONE
                    .min_width(WIDTH_MIN)
                    .max_width(WIDTH_WIDE)
                    .min_height(200.0)
                    .max_height(900.0);
                return get_popup(settings)
                    .chain(widget::text_input::focus(self.search_id.clone()));
            }
            Message::Closed(id) => {
                if self.popup == Some(id) {
                    self.dragging = false;
                    self.popup = None;
                }
            }
            Message::Tick => {
                let now = Instant::now();
                if self.flash.as_ref().is_some_and(|(_, until)| now >= *until) {
                    self.flash = None;
                }
                if self.undo.as_ref().is_some_and(|undo| now >= undo.until) {
                    self.undo = None;
                }
                // Fin de la pause temporaire : reprise sans importer les copies
                // faites entre-temps, l’époque du moniteur ayant changé.
                if let Some(end) = self.pause_until
                    && now >= end
                {
                    return self.update(Message::Resume);
                }
                // Preserve the source widget and card order until a drag ends.
                if self.dragging {
                    return Task::none();
                }
                let events: Vec<_> = self.monitor.events.try_iter().collect();
                for event in events {
                    match event {
                        Event::Clip(clip, epoch)
                            if !self.monitor.paused()
                                && epoch == self.monitor.epoch.load(Ordering::SeqCst) =>
                        {
                            if let Some(store) = &mut self.store {
                                match store.insert(&clip) {
                                    Ok(()) => {
                                        self.status = tr!("Historique local · Clique sur une carte pour la copier", "Local history · click a card to copy it").into();
                                        if !self.clips.iter().any(|c| c.id == clip.id) {
                                            self.clips.push(clip);
                                        }
                                        self.refresh();
                                    }
                                    Err(e) => self.status = e,
                                }
                            }
                        }
                        Event::Status(result) => {
                            self.connected = result.is_ok();
                            if let Err(e) = result {
                                self.status = e;
                            } else if self.store.is_some() {
                                self.status = tr!(
                                    "Historique local · Capture active",
                                    "Local history · capture active"
                                )
                                .into();
                            }
                        }
                        Event::Rejected(error) => self.status = error,
                        _ => {}
                    }
                }
                let mut bytes = [0; 16];
                if let Some(instance) = &self.instance
                    && let Ok(6) = instance.socket.recv(&mut bytes)
                    && &bytes[..6] == b"toggle"
                {
                    return self.update(Message::Toggle);
                }
            }
            Message::Search(query) => {
                self.query = query;
                self.reset();
            }
            Message::Filter(kind) => {
                self.kind = kind;
                self.reset();
            }
            Message::Favorites(favorites) => {
                self.favorites = favorites;
                self.category.clear();
                self.reset();
            }
            Message::Category(category) => {
                self.category = category;
                self.favorites = false;
                self.reset();
            }
            Message::Copy(id) => return self.copy(&id),
            Message::CopyPlain(id) => return self.copy_plain(&id),
            Message::PlainSelected => {
                if let Some(id) = self.target() {
                    return self.copy_plain(&id);
                }
            }
            Message::Copied(result) => match result {
                Ok(()) => {
                    self.status = tr!(
                        "Copié · Colle avec Ctrl+V dans ton application",
                        "Copied · press Ctrl+V in your application"
                    )
                    .into();
                    self.flash(tr!("Copié dans le presse-papiers", "Copied to clipboard"));
                    // La fermeture attend que la confirmation ait été visible.
                    return Task::perform(
                        async { tokio::time::sleep(Duration::from_millis(450)).await },
                        |()| cosmic::Action::App(Message::FinishCopy),
                    );
                }
                Err(e) => {
                    self.copying = false;
                    self.status = tr_format!("Échec de la copie : {e}", "Copy failed: {e}");
                }
            },
            Message::FinishCopy => {
                if self.settings.keep_open && self.settings.paste_mode == 0 {
                    self.copying = false;
                    return Task::none();
                }
                let close = self.popup.take().map_or_else(Task::none, destroy_popup);
                if self.settings.paste_mode != 0 {
                    let terminal = self.settings.paste_mode == 2;
                    return close.chain(Task::perform(
                        async move {
                            tokio::time::sleep(Duration::from_millis(350)).await;
                            terminal
                        },
                        |terminal| cosmic::Action::App(Message::PasteReady(terminal)),
                    ));
                }
                self.copying = false;
                return close;
            }
            Message::Pin(id) => {
                if let Some(clip) = self.clips.iter().find(|c| c.id == id) {
                    let pinned = !clip.pinned;
                    self.write(|s| s.pin(&id, pinned));
                }
            }
            Message::Delete(id) => {
                if let Some(clip) = self.clips.iter().find(|c| c.id == id).cloned() {
                    self.monitor.invalidate();
                    if self.write(|store| store.delete(&id)) {
                        self.undo = Some(Undo {
                            clip,
                            until: Instant::now() + Duration::from_secs(UNDO_SECONDS),
                        });
                        self.flash(tr!("Copie supprimée", "Clip deleted"));
                        if self.detail.as_ref() == Some(&id) {
                            self.detail = None;
                        }
                    }
                }
            }
            Message::Undo => {
                if self
                    .undo
                    .as_ref()
                    .is_some_and(|u| Instant::now() >= u.until)
                {
                    self.undo = None;
                }
                if let Some(undo) = &self.undo {
                    if let Some(store) = &mut self.store {
                        match store.restore(&undo.clip) {
                            Ok(inserted) => {
                                self.undo = None;
                                self.refresh();
                                self.flash(if inserted {
                                    tr!("Suppression annulée", "Deletion undone")
                                } else {
                                    tr!(
                                        "Cette copie est déjà dans l’historique",
                                        "This clip is already in history"
                                    )
                                });
                            }
                            Err(e) => {
                                self.flash = None;
                                self.status = e;
                            }
                        }
                    }
                }
            }
            Message::Detail(id) => {
                self.category_edit = id
                    .as_ref()
                    .and_then(|id| self.clips.iter().find(|c| &c.id == id))
                    .map(|c| c.category.clone())
                    .unwrap_or_default();
                self.detail = id;
            }
            Message::EditCategory(value) => self.category_edit = value.chars().take(40).collect(),
            Message::SaveCategory => {
                if let Some(id) = self.detail.clone() {
                    let category = self.category_edit.trim().to_string();
                    if self.write(|s| s.category(&id, &category)) {
                        self.status = tr!("Catégorie enregistrée", "Collection saved").into();
                    }
                }
            }
            Message::PauseMenu(open) => {
                self.pause_menu = open;
                self.settings_open = false;
            }
            Message::PauseFor(minutes) => {
                self.pause_menu = false;
                if self.store.is_some() && !self.monitor.paused() {
                    self.monitor.toggle_pause();
                    self.pause_until =
                        minutes.map(|m| Instant::now() + Duration::from_secs(m * 60));
                    self.status = match minutes {
                        Some(m) => tr_format!(
                            "Capture suspendue pendant {m} minutes",
                            "Capture paused for {m} minutes"
                        ),
                        None => tr!(
                            "Capture suspendue jusqu’à reprise manuelle",
                            "Capture paused until resumed"
                        )
                        .into(),
                    };
                }
            }
            Message::Resume => {
                self.pause_until = None;
                if self.store.is_some() && self.monitor.paused() {
                    // L’incrément rend l’époque paire : les copies faites pendant la
                    // pause ne sont jamais importées après coup.
                    self.monitor.toggle_pause();
                    self.status = if self.connected {
                        tr!("Capture active", "Capture active").into()
                    } else {
                        tr!(
                            "Connexion à Wayland indisponible",
                            "Wayland connection unavailable"
                        )
                        .into()
                    };
                    self.flash(tr!("Capture reprise", "Capture resumed"));
                }
            }
            Message::AskClear => self.clear_confirm = true,
            Message::CancelClear => self.clear_confirm = false,
            Message::Clear => {
                self.monitor.invalidate();
                self.undo = None;
                self.write(Store::clear_unpinned);
                self.clear_confirm = false;
                self.reset();
            }
            Message::Page(next) => {
                let page_size = self.page_size();
                if next {
                    self.page =
                        (self.page + 1).min(self.filtered().len().saturating_sub(1) / page_size);
                } else {
                    self.page = self.page.saturating_sub(1);
                }
                self.selected = 0;
            }
            Message::Move(next) => self.step(if next { 1 } else { -1 }),
            // En liste compacte une colonne, un déplacement de ligne vaut un déplacement
            // d’élément : la navigation reste la même dans les deux densités.
            Message::MoveRow(next) => {
                let step = self.columns() as isize;
                self.step(if next { step } else { -step });
            }
            Message::Enter => {
                if let Some(id) = self.detail.clone() {
                    return self.copy(&id);
                }
                return self.update(Message::Choose(self.selected));
            }
            Message::Choose(index) => {
                let page_size = self.page_size();
                if index >= page_size {
                    return Task::none();
                }
                if let Some(clip) = self.filtered().get(self.page * page_size + index) {
                    let id = clip.id.clone();
                    return self.copy(&id);
                }
            }
        }
        Task::none()
    }
    fn style(&self) -> Option<iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic::Application;

    #[test]
    fn deleted_source_cancels_pending_ocr_without_reinserting_text() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        let source = app
            .clips
            .iter()
            .find(|c| c.kind == Kind::Image)
            .unwrap()
            .id
            .clone();
        let _ = app.update(Message::Delete(source.clone()));
        let count = app.clips.len();
        app.ocr_busy = true;
        let _ = app.update(Message::OcrDone(source, Ok("Extracted text".into())));
        assert_eq!(app.clips.len(), count);
        assert!(!app.ocr_busy);
    }

    #[test]
    fn ocr_result_is_saved_and_opened_without_copying() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        let source = app
            .clips
            .iter()
            .find(|c| c.kind == Kind::Image)
            .unwrap()
            .id
            .clone();
        let _ = app.update(Message::OcrDone(
            source,
            Ok("Texte reconnu à vérifier".into()),
        ));
        let result = app
            .clips
            .iter()
            .find(|c| c.text == "Texte reconnu à vérifier")
            .unwrap();
        assert_eq!(app.detail.as_ref(), Some(&result.id));
        assert!(!app.copying);
    }

    #[test]
    fn undoing_a_deletion_restores_the_favorite_and_its_category() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        let clip = app.clips.iter().find(|c| c.pinned).unwrap().clone();
        let count = app.clips.len();
        let _ = app.update(Message::Delete(clip.id.clone()));
        assert!(!app.clips.iter().any(|c| c.id == clip.id));
        assert!(app.undo.is_some());
        let _ = app.update(Message::Undo);
        assert_eq!(app.clips.len(), count);
        let restored = app.clips.iter().find(|c| c.id == clip.id).unwrap();
        assert!(restored.pinned);
        assert_eq!(restored.category, clip.category);
        assert_eq!(restored.timestamp, clip.timestamp);
        // Une seconde annulation ne réinsère rien : la fenêtre est consommée.
        let _ = app.update(Message::Undo);
        assert_eq!(app.clips.len(), count);
    }

    #[test]
    fn a_narrow_panel_reduces_the_grid_instead_of_hiding_cards() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        assert_eq!(app.columns(), 4);
        assert_eq!(app.page_size(), 8);
        let _ = app.update(Message::Viewport(520.0));
        assert_eq!(app.columns(), 2);
        assert_eq!(app.page_size(), 4);
        // Un raccourci hors de la page reste sans effet, dans les deux densités.
        let outside = app.page_size();
        let _ = app.update(Message::Choose(outside));
        assert!(!app.status.contains("Démonstration"));
        let _ = app.update(Message::Choose(0));
        assert!(app.status.contains("Démonstration"));
    }

    #[test]
    fn compact_density_keeps_one_column_and_bounded_navigation() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        let _ = app.update(Message::Density);
        assert_eq!(app.settings.density, Density::Compact);
        assert_eq!(app.columns(), 1);
        // Une ligne vaut un élément en liste : même déplacement qu’en grille.
        let _ = app.update(Message::MoveRow(true));
        assert_eq!(app.selected, 1);
        for _ in 0..20 {
            let _ = app.update(Message::MoveRow(true));
        }
        assert!(app.selected < app.filtered().len().min(app.page_size()));
        for _ in 0..20 {
            let _ = app.update(Message::Move(false));
        }
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn plain_text_copy_refuses_an_image_before_touching_the_clipboard() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        let image = app
            .clips
            .iter()
            .find(|c| c.kind == Kind::Image)
            .unwrap()
            .id
            .clone();
        // Hors démonstration, le refus vient du contenu, pas du mode.
        app.demo = false;
        let _ = app.update(Message::CopyPlain(image));
        assert!(app.status.contains("image"));
        assert!(!app.copying);
    }

    #[test]
    fn a_temporary_pause_resumes_on_its_own_deadline() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        // Le moniteur de démonstration démarre à l’arrêt : on part d’une capture active.
        app.monitor.toggle_pause();
        assert!(!app.monitor.paused());
        let _ = app.update(Message::PauseFor(Some(5)));
        assert!(app.monitor.paused());
        assert!(app.pause_until.is_some_and(|end| end > Instant::now()));
        // Échéance atteinte : la reprise vient du minuteur, pas de l’utilisateur.
        app.pause_until = Some(Instant::now() - Duration::from_secs(1));
        let _ = app.update(Message::Tick);
        assert!(!app.monitor.paused());
        assert!(app.pause_until.is_none());
    }
    #[test]
    fn expired_undo_and_clear_cannot_restore_data() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        let id = app.clips[0].id.clone();
        let _ = app.update(Message::Delete(id.clone()));
        app.undo.as_mut().unwrap().until = Instant::now() - Duration::from_secs(1);
        let _ = app.update(Message::Undo);
        assert!(!app.clips.iter().any(|c| c.id == id));
        let id = app.clips[0].id.clone();
        let _ = app.update(Message::Delete(id));
        let _ = app.update(Message::Clear);
        assert!(app.undo.is_none());
    }
    #[test]
    fn cycling_retention_does_not_delete_before_apply() {
        let (mut app, _) = App::init(cosmic::Core::default(), Mode::Preview);
        let old = Clip::new("text/plain".into(), b"old retention fixture".to_vec(), 1).unwrap();
        app.store.as_mut().unwrap().insert(&old).unwrap();
        app.refresh();
        let _ = app.update(Message::Retention);
        assert_eq!(app.settings.retention_days, 0);
        assert!(app.clips.iter().any(|c| c.id == old.id));
        let _ = app.update(Message::ApplyRetention);
        assert!(!app.clips.iter().any(|c| c.id == old.id));
    }
}
