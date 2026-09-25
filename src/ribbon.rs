//! Expanded applet: a short, preview-first horizontal shelf.
use super::*;

impl App {
    pub(super) fn ribbon_visible(&self) -> bool {
        self.is_popup()
            && !self.compact_popup()
            && !self.templates_open
            && !self.settings_open
            && !self.collections_open
            && !self.pause_menu
            && !self.clear_confirm
            && self.detail.is_none()
    }

    fn ribbon_count(&self, kind: Option<Kind>) -> usize {
        self.clips
            .iter()
            .filter(|clip| {
                clip.matches_with_ocr(
                    &self.query,
                    kind,
                    self.favorites,
                    &self.category,
                    self.image_index
                        .get(&clip.id)
                        .map(String::as_str)
                        .unwrap_or(""),
                )
            })
            .count()
    }

    fn ribbon_card<'a>(&self, clip: &'a Clip, index: usize, width: f32) -> Element<'a, Message> {
        let metadata = widget::row([])
            .push(skin::icon(clip.kind.icon()).icon().size(12))
            .push(widget::text(model::age(clip.timestamp)).size(11))
            .push(widget::Space::new().width(Length::Fill))
            .push(
                widget::text(format!(
                    "{}⌃{}",
                    if clip.pinned { "★ · " } else { "" },
                    index + 1
                ))
                .size(11),
            )
            .spacing(5)
            .align_y(iced::Alignment::Center);
        let body = widget::column([])
            .push(self.preview(clip, 132.0))
            .push(widget::container(metadata).padding([0, 4]))
            .spacing(8);
        let copy = widget::button::custom(body)
            .on_press(Message::Copy(clip.id.clone()))
            .class(skin::button(self.selected == index, 12.0, true))
            .padding(8)
            .width(width);
        // The preview is also available from the selected-item toolbar, so
        // right-click is never the only way to reach destructive/organizing actions.
        iced::widget::mouse_area(copy)
            .on_right_press(Message::Detail(Some(clip.id.clone())))
            .into()
    }

    pub(super) fn view_ribbon(&self) -> Element<'_, Message> {
        let paused = self.monitor.paused();
        let header = widget::row([])
            .push(skin::hint(
                widget::button::custom(skin::brand_icon().icon().size(24))
                    .on_press(Message::TogglePopupSize),
                tr!("Vue compacte", "Compact view"),
            ))
            .push(
                widget::search_input(
                    tr!("Rechercher une copie…", "Search clipboard history…"),
                    &self.query,
                )
                .id(self.search_id.clone())
                .on_input(Message::Search)
                .on_submit(|_| Message::Enter)
                .padding(8)
                .width(Length::Fill),
            )
            .push(skin::hint(
                widget::button::icon(skin::icon(if paused {
                    "media-playback-start-symbolic"
                } else {
                    "media-playback-pause-symbolic"
                }))
                .on_press(if paused {
                    Message::Resume
                } else {
                    Message::PauseMenu(true)
                }),
                if paused {
                    tr!("Reprendre la capture", "Resume capture")
                } else {
                    tr!("Mettre en pause", "Pause capture")
                },
            ))
            .push(skin::hint(
                widget::button::icon(skin::icon("preferences-system-symbolic"))
                    .on_press(Message::Settings(true)),
                tr!("Préférences", "Preferences"),
            ))
            .push(skin::hint(
                widget::button::icon(skin::icon("window-close-symbolic"))
                    .on_press(Message::CloseView),
                tr!("Fermer", "Close"),
            ))
            .spacing(6)
            .align_y(iced::Alignment::Center);

        let mut tabs = widget::row([])
            .spacing(4)
            .push(
                widget::button::text(format!(
                    "{} {}",
                    tr!("Historique", "History"),
                    self.clips.len()
                ))
                .class(skin::button(
                    !self.favorites && self.category.is_empty(),
                    8.0,
                    false,
                ))
                .on_press(Message::Favorites(false)),
            )
            .push(
                widget::button::text(format!(
                    "{} {}",
                    tr!("Favoris", "Favorites"),
                    self.clips.iter().filter(|c| c.pinned).count()
                ))
                .class(skin::button(self.favorites, 8.0, false))
                .on_press(Message::Favorites(true)),
            );
        for category in &self.collections {
            let count = self
                .clips
                .iter()
                .filter(|c| c.category == *category)
                .count();
            tabs = tabs.push(
                widget::button::text(format!(
                    "{} {count}",
                    category.chars().take(24).collect::<String>()
                ))
                .class(skin::button(self.category == *category, 8.0, false))
                .on_press(Message::Category(category.clone())),
            );
        }
        tabs = tabs
            .push(skin::hint(
                widget::button::text("+").on_press(Message::Collections(true)),
                tr!("Gérer les collections", "Manage collections"),
            ))
            .push(
                widget::button::text(tr!("Modèles", "Templates"))
                    .on_press(Message::Templates(true)),
            );
        let tabs = widget::scrollable(tabs)
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::default(),
            ))
            .width(Length::Fill);
        let mut filters = widget::row([]).spacing(2);
        for kind in std::iter::once(None).chain(Kind::ALL.into_iter().map(Some)) {
            filters = filters.push(
                widget::button::text(format!(
                    "{} {}",
                    kind.map_or(tr!("Tout", "All"), Kind::label),
                    self.ribbon_count(kind)
                ))
                .class(skin::button(self.kind == kind, 8.0, false))
                .on_press(Message::Filter(kind)),
            );
        }
        let filters = widget::scrollable(filters)
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::default(),
            ))
            .width(Length::Fill);
        let filtered = self.filtered();
        let page_size = self.page_size();
        let pages = filtered.len().max(1).div_ceil(page_size);
        let width = (self.width() - 24.0 - (page_size - 1) as f32 * 10.0) / page_size as f32;
        let mut cards = widget::row([]).spacing(10);
        for (index, clip) in filtered
            .iter()
            .skip(self.page * page_size)
            .take(page_size)
            .enumerate()
        {
            cards = cards.push(self.ribbon_card(clip, index, width));
        }
        let shelf: Element<'_, Message> = if filtered.is_empty() {
            widget::container(
                widget::text(if self.query.is_empty() {
                    tr!("Tout commence par une copie.", "It starts with a copy.")
                } else {
                    tr!("Aucun résultat.", "No results.")
                })
                .size(16),
            )
            .center_x(Length::Fill)
            .center_y(174)
            .into()
        } else {
            widget::scrollable(cards)
                .direction(iced::widget::scrollable::Direction::Horizontal(
                    iced::widget::scrollable::Scrollbar::default(),
                ))
                .width(Length::Fill)
                .height(184)
                .into()
        };
        let selected = filtered.get(self.page * page_size + self.selected);
        let footer = widget::row([])
            .push(
                widget::button::text(tr!("Aperçu / actions", "Preview / actions"))
                    .on_press_maybe(selected.map(|c| Message::Detail(Some(c.id.clone())))),
            )
            .push(
                widget::button::text(tr!("Fenêtre séparée", "Separate window"))
                    .on_press(Message::OpenHistory),
            )
            .push(widget::Space::new().width(Length::Fill))
            .push(
                widget::button::text("←")
                    .on_press_maybe((self.page > 0).then_some(Message::Page(false))),
            )
            .push(widget::text(format!("{} / {pages}", self.page + 1)).size(11))
            .push(
                widget::button::text("→")
                    .on_press_maybe((self.page + 1 < pages).then_some(Message::Page(true))),
            )
            .align_y(iced::Alignment::Center)
            .spacing(4);
        let status = self
            .flash
            .as_ref()
            .map(|(s, _)| s.as_str())
            .unwrap_or(&self.status);
        let mut content = widget::column([])
            .push(header)
            .push(tabs)
            .push(filters)
            .push(shelf)
            .push(footer)
            .push(widget::text(status).size(11))
            .spacing(6);
        if self.undo.is_some() {
            content = content.push(
                widget::button::text(tr!("Annuler la suppression", "Undo deletion"))
                    .on_press(Message::Undo),
            );
        }
        let view = self
            .core
            .applet
            .popup_container(widget::container(content).padding(12).width(self.width()))
            .limits(
                Limits::NONE
                    .min_width(WIDTH_MIN)
                    .max_width(self.ideal_width())
                    .max_height(500.0),
            );
        let mut theme = self.application_theme.clone();
        theme.transparent = cosmic::theme::active().transparent;
        iced::widget::themer(Some(theme), view)
            .text_color(|theme| theme.cosmic().on_bg_color().into())
            .into()
    }
}
