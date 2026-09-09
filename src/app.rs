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
    storage::Store,
};
use std::{collections::HashMap, sync::atomic::Ordering, time::Duration};

const PAGE_SIZE: usize = 8;
const WIDTH: f32 = 940.0;
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
    paste_mode: u8,
    ocr_busy: bool,
    ocr_language: &'static str,
    dragging: bool,
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
    Copied(Result<(), String>),
    Pin(String),
    Delete(String),
    Detail(Option<String>),
    EditCategory(String),
    SaveCategory,
    Pause,
    AskClear,
    CancelClear,
    Clear,
    Page(bool),
    Move(bool),
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
    fn reset(&mut self) {
        self.page = 0;
        self.selected = 0;
        self.clear_confirm = false;
    }
    fn refresh(&mut self) {
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
        self.page = self
            .page
            .min(self.filtered().len().saturating_sub(1) / PAGE_SIZE);
        self.selected = self
            .selected
            .min(self.filtered().len().saturating_sub(1).min(PAGE_SIZE - 1));
    }
    fn write(&mut self, operation: impl FnOnce(&Store) -> Result<(), String>) {
        if let Some(store) = &self.store
            && let Err(e) = operation(store)
        {
            self.status = e;
            return;
        }
        self.refresh();
    }
    fn copy(&mut self, id: &str) -> Task<cosmic::Action<Message>> {
        if self.dragging {
            return Task::none();
        }
        if self.demo {
            self.status = "Démonstration : copie désactivée".into();
            return Task::none();
        }
        if self.copying {
            return Task::none();
        }
        if let Some(clip) = self.clips.iter().find(|c| c.id == id).cloned() {
            self.copying = true;
            self.status = "Copie…".into();
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
                .unwrap_or("Lien web");
            widget::container(
                widget::column([])
                    .push(skin::icon("insert-link-symbolic").icon().size(24))
                    .push(
                        widget::text(domain.chars().take(23).collect::<String>())
                            .size(17)
                            .class(skin::kind_color(Kind::Link)),
                    )
                    .push(widget::text("Lien enregistré").size(11).class(skin::MUTED))
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
                        widget::text(format!("{count} fichier(s)"))
                            .size(18)
                            .class(skin::kind_color(Kind::Files)),
                    )
                    .push(
                        widget::text("Emplacements d’origine")
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
    fn card<'a>(&self, clip: &'a Clip, index: usize) -> Element<'a, Message> {
        let header = widget::row([])
            .push(skin::icon(clip.kind.icon()).icon().size(14))
            .push(
                widget::text(clip.kind.label())
                    .size(11)
                    .class(skin::kind_color(clip.kind)),
            )
            .push(widget::Space::new().width(Length::Fill))
            .push(
                widget::text(format!("⌃{}", index + 1))
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
        let drag = if self.demo {
            drag
        } else {
            drag.drag_content(move || payload.clone())
        };
        let actions = widget::row([])
            .push(drag)
            .push(
                widget::button::text(if clip.pinned { "★" } else { "☆" })
                    .class(skin::button(clip.pinned, 6.0, false))
                    .on_press(Message::Pin(clip.id.clone())),
            )
            .push(
                widget::button::text("Aperçu")
                    .class(skin::button(false, 6.0, false))
                    .on_press(Message::Detail(Some(clip.id.clone()))),
            )
            .push(widget::Space::new().width(Length::Fill))
            .push(
                widget::button::icon(skin::icon("edit-delete-symbolic"))
                    .class(skin::button(false, 6.0, false))
                    .on_press(Message::Delete(clip.id.clone())),
            )
            .spacing(2);
        widget::column([])
            .push(copy)
            .push(actions)
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
                .ok_or("Dossier utilisateur introuvable".into())
                .and_then(|p| Store::open(&p.join("nebula-paste/history.sqlite3")))
        };
        let (store, error) = match result {
            Ok(s) => (Some(s), None),
            Err(e) => (None, Some(e)),
        };
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
            status: error.unwrap_or("Connexion à Wayland…".into()),
            connected: false,
            clear_confirm: false,
            copying: false,
            search_id: iced::widget::Id::unique(),
            paste_mode: 0,
            ocr_busy: false,
            ocr_language: "fra+eng",
            dragging: false,
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
                "edit-paste-symbolic"
            })
            .on_press(Message::Toggle)
            .into()
    }
    fn view_window(&self, _: Id) -> Element<'_, Message> {
        let title = widget::row([])
            .push(skin::icon("edit-paste-symbolic").icon().size(20))
            .push(
                widget::column([])
                    .push(widget::text("Nebula Paste").size(23).class(skin::TEXT))
                    .push(
                        widget::text("Tes idées, toujours à portée de main.")
                            .size(12)
                            .class(skin::MUTED),
                    )
                    .spacing(3),
            )
            .push(widget::Space::new().width(Length::Fill))
            .push(
                widget::button::text(match self.paste_mode {
                    1 => "Coller · Ctrl+V",
                    2 => "Coller · Terminal",
                    _ => "Copier seulement",
                })
                .class(skin::button(self.paste_mode != 0, 8.0, false))
                .on_press(Message::PasteMode),
            )
            .push(
                widget::button::text(if self.monitor.paused() && !self.demo {
                    "Reprendre"
                } else {
                    "Pause"
                })
                .class(skin::button(false, 8.0, false))
                .on_press(Message::Pause),
            )
            .push(
                widget::button::icon(skin::icon("window-close-symbolic"))
                    .class(skin::button(false, 8.0, false))
                    .on_press(Message::Toggle),
            )
            .spacing(10)
            .align_y(iced::Alignment::Center);
        let mut layout = widget::column([]).push(title).spacing(14);
        if let Some(clip) = self
            .detail
            .as_ref()
            .and_then(|id| self.clips.iter().find(|c| &c.id == id))
        {
            let body: Element<'_, Message> = if clip.kind == Kind::Image || clip.kind == Kind::Color
            {
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
                    widget::button::text("← Historique")
                        .class(skin::button(false, 8.0, false))
                        .on_press(Message::Detail(None)),
                )
                .push(widget::text(&clip.title).size(18).class(skin::TEXT))
                .push(body)
                .push(
                    widget::text(format!(
                        "{} · {} octets · {}",
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
                                "Catégorie : Travail, Commandes…",
                                &self.category_edit,
                            )
                            .style(skin::input())
                            .padding(10)
                            .on_input(Message::EditCategory)
                            .on_submit(|_| Message::SaveCategory),
                        )
                        .push(
                            widget::button::standard("Enregistrer")
                                .class(skin::button(true, 8.0, false))
                                .on_press(Message::SaveCategory),
                        )
                        .spacing(8),
                )
                .push(
                    widget::button::suggested(if self.paste_mode == 0 {
                        "Copier le contenu"
                    } else {
                        "Copier et coller"
                    })
                    .class(skin::button(true, 8.0, true))
                    .on_press(Message::Copy(clip.id.clone())),
                );
            if clip.kind == Kind::Image {
                layout = layout.push(
                    widget::row([])
                        .push(
                            widget::button::text(if self.ocr_busy {
                                "Lecture de l’image…"
                            } else {
                                "Extraire le texte · OCR"
                            })
                            .class(skin::button(true, 8.0, true))
                            .on_press_maybe(
                                (!self.ocr_busy).then(|| Message::Ocr(clip.id.clone())),
                            ),
                        )
                        .push(
                            widget::button::text(match self.ocr_language {
                                "fra" => "Français",
                                "eng" => "Anglais",
                                _ => "Français + anglais",
                            })
                            .class(skin::button(false, 8.0, false))
                            .on_press(Message::OcrLanguage),
                        )
                        .push(widget::Space::new().width(Length::Fill))
                        .push(
                            widget::text("OCR intégré · hors ligne")
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
                    widget::button::text("Historique")
                        .class(skin::button(
                            !self.favorites && self.category.is_empty(),
                            8.0,
                            false,
                        ))
                        .on_press(Message::Favorites(false)),
                )
                .push(
                    widget::button::text("★ Favoris")
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
                            widget::search_input("Rechercher une copie…", &self.query)
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
                    widget::button::text(kind.map_or("Tout", Kind::label))
                        .class(skin::button(self.kind == kind, 7.0, false))
                        .on_press(Message::Filter(kind)),
                );
            }
            let filtered = self.filtered();
            filters = filters.push(widget::Space::new().width(Length::Fill)).push(
                widget::text(format!("{} copies", filtered.len()))
                    .size(12)
                    .class(skin::MUTED),
            );
            layout = layout.push(filters.align_y(iced::Alignment::Center));
            let page: Vec<_> = filtered
                .iter()
                .skip(self.page * PAGE_SIZE)
                .take(PAGE_SIZE)
                .collect();
            let mut grid = widget::column([]).spacing(14);
            if page.is_empty() {
                grid=grid.push(widget::container(widget::column([])
                .push(widget::text(if self.query.is_empty() {"Tout commence par une copie."} else {"Aucun résultat."}).size(22).class(skin::TEXT))
                .push(widget::text("Textes, images, liens… retrouve-les ici, quand tu en as besoin.").size(13).class(skin::MUTED)).spacing(12)).center_x(Length::Fill).center_y(200));
            }
            for (row_index, chunk) in page.chunks(4).enumerate() {
                let mut row = widget::row([]).spacing(12);
                for (col, clip) in chunk.iter().enumerate() {
                    row = row.push(self.card(clip, row_index * 4 + col));
                }
                for _ in chunk.len()..4 {
                    row = row.push(widget::Space::new().width(Length::FillPortion(1)));
                }
                grid = grid.push(row);
            }
            layout = layout.push(widget::scrollable(grid).height(if page.len() > 4 {
                480
            } else {
                280
            }));
            let page_count = filtered.len().max(1).div_ceil(PAGE_SIZE);
            layout = layout.push(
                widget::row([])
                    .push(
                        widget::text("⠿ Glisser vers une application · Ctrl+1…8 : choisir")
                            .size(11)
                            .class(skin::MUTED),
                    )
                    .push(widget::Space::new().width(Length::Fill))
                    .push(
                        widget::button::text("Vider…")
                            .class(skin::button(false, 7.0, false))
                            .on_press(Message::AskClear),
                    )
                    .push(
                        widget::button::text("←")
                            .class(skin::button(false, 7.0, false))
                            .on_press_maybe((self.page > 0).then_some(Message::Page(false))),
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
                                (self.page + 1 < page_count).then_some(Message::Page(true)),
                            ),
                    )
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
            );
        }
        layout = layout.push(
            widget::container(
                widget::text(if self.demo && self.status.starts_with("Connexion") {
                    "Démonstration · historique isolé"
                } else {
                    &self.status
                })
                .size(12)
                .class(skin::MUTED),
            )
            .padding([8, 10])
            .width(Length::Fill)
            .class(skin::surface(skin::PREVIEW, 8.0, skin::LINE)),
        );
        if self.clear_confirm {
            layout = layout.push(
                widget::row([])
                    .push(widget::text("Effacer l’historique hors favoris ?").class(skin::TEXT))
                    .push(
                        widget::button::text("Annuler")
                            .class(skin::button(false, 8.0, false))
                            .on_press(Message::CancelClear),
                    )
                    .push(
                        widget::button::text("Effacer")
                            .class(skin::button(true, 8.0, false))
                            .on_press(Message::Clear),
                    )
                    .spacing(8),
            );
        }
        let content = widget::container(layout)
            .width(WIDTH)
            .padding(18)
            .class(skin::surface(skin::BG, 16.0, skin::LINE));
        if self.demo {
            content.into()
        } else {
            self.core
                .applet
                .popup_container(content)
                .limits(
                    Limits::NONE
                        .min_width(WIDTH)
                        .max_width(WIDTH)
                        .max_height(850.0),
                )
                .into()
        }
    }
    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions =
            vec![iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)];
        if self.popup.is_some() || self.demo {
            subscriptions.push(iced::event::listen_with(|event, status, _| {
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
                    Key::Character("f") if mods.control() => Some(Message::FocusSearch),
                    Key::Named(Named::ArrowRight) if mods.alt() => Some(Message::Move(true)),
                    Key::Named(Named::ArrowLeft) if mods.alt() => Some(Message::Move(false)),
                    Key::Character(c) if mods.control() => c
                        .parse::<usize>()
                        .ok()
                        .filter(|n| (1..=PAGE_SIZE).contains(n))
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
                self.paste_mode = (self.paste_mode + 1) % 3;
                self.status = match self.paste_mode {
                    1 => "Collage direct · Ctrl+V envoyé à l’application active après fermeture. Nécessite wtype.",
                    2 => "Mode terminal · Ctrl+Maj+V envoyé après fermeture. Nécessite wtype.",
                    _ => "Copie seule · colle ensuite avec Ctrl+V.",
                }.into();
            }
            Message::OcrLanguage => {
                self.ocr_language = match self.ocr_language {
                    "fra+eng" => "fra",
                    "fra" => "eng",
                    _ => "fra+eng",
                };
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
                    self.status = "Reconnaissance locale en cours…".into();
                    let language = self.ocr_language;
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
                                    self.status =
                                        "Texte extrait · vérifie le résultat avant de le copier."
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
                    Ok(()) => self.status = "Raccourci de collage envoyé.".into(),
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
                let mut settings = self
                    .core
                    .applet
                    .get_popup_settings(parent, id, None, None, None);
                settings.positioner.size_limits = Limits::NONE
                    .min_width(WIDTH)
                    .max_width(WIDTH)
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
                // Preserve the source widget and card order until a drag ends.
                if self.dragging {
                    return Task::none();
                }
                let events: Vec<_> = self.monitor.events.try_iter().collect();
                for event in events {
                    match event {
                        Event::Clip(clip, epoch)
                            if epoch == self.monitor.epoch.load(Ordering::SeqCst) =>
                        {
                            if let Some(store) = &mut self.store {
                                match store.insert(&clip) {
                                    Ok(()) => {
                                        self.status = "Historique local · Clique sur une carte pour la copier".into();
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
                                self.status = "Historique local · Capture active".into();
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
            Message::Copied(result) => {
                self.copying = false;
                match result {
                    Ok(()) => {
                        self.status = "Copié · Colle avec Ctrl+V dans ton application".into();
                        let close = self.popup.take().map_or_else(Task::none, destroy_popup);
                        if self.paste_mode != 0 {
                            self.copying = true;
                            let terminal = self.paste_mode == 2;
                            return close.chain(Task::perform(
                                async move {
                                    tokio::time::sleep(Duration::from_millis(350)).await;
                                    terminal
                                },
                                |terminal| cosmic::Action::App(Message::PasteReady(terminal)),
                            ));
                        }
                        return close;
                    }
                    Err(e) => self.status = format!("Échec de la copie : {e}"),
                }
            }
            Message::Pin(id) => {
                if let Some(clip) = self.clips.iter().find(|c| c.id == id) {
                    let pinned = !clip.pinned;
                    self.write(|s| s.pin(&id, pinned));
                }
            }
            Message::Delete(id) => {
                self.monitor.invalidate();
                self.write(|s| s.delete(&id));
                if self.detail.as_ref() == Some(&id) {
                    self.detail = None;
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
                    self.write(|s| s.category(&id, &category));
                    self.status = "Catégorie enregistrée".into();
                }
            }
            Message::Pause => {
                if self.store.is_some() {
                    self.monitor.toggle_pause();
                    self.status = if self.monitor.paused() {
                        "Capture en pause".into()
                    } else if self.connected {
                        "Capture active".into()
                    } else {
                        "Connexion à Wayland indisponible".into()
                    };
                }
            }
            Message::AskClear => self.clear_confirm = true,
            Message::CancelClear => self.clear_confirm = false,
            Message::Clear => {
                self.monitor.invalidate();
                self.write(Store::clear_unpinned);
                self.clear_confirm = false;
                self.reset();
            }
            Message::Page(next) => {
                if next {
                    self.page =
                        (self.page + 1).min(self.filtered().len().saturating_sub(1) / PAGE_SIZE);
                } else {
                    self.page = self.page.saturating_sub(1);
                }
                self.selected = 0;
            }
            Message::Move(next) => {
                let count = self
                    .filtered()
                    .len()
                    .saturating_sub(self.page * PAGE_SIZE)
                    .min(PAGE_SIZE);
                self.selected = if next {
                    (self.selected + 1).min(count.saturating_sub(1))
                } else {
                    self.selected.saturating_sub(1)
                };
            }
            Message::Enter => {
                if let Some(id) = self.detail.clone() {
                    return self.copy(&id);
                }
                return self.update(Message::Choose(self.selected));
            }
            Message::Choose(index) => {
                if index >= PAGE_SIZE {
                    return Task::none();
                }
                if let Some(clip) = self.filtered().get(self.page * PAGE_SIZE + index) {
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
}
