//! COSMIC theme roles; brand artwork and clipboard colour swatches keep their own colours.
use cosmic::{
    iced::{Border, Color},
    theme, widget,
};

pub const TEXT: theme::Text = theme::Text::Default;
pub const MUTED: theme::Text = theme::Text::Default;
pub const ACCENT: theme::Text = theme::Text::Accent;

pub fn brand_icon() -> widget::icon::Handle {
    widget::icon::from_svg_bytes(
        include_bytes!("../resources/io.github.nebulapaste.NebulaPaste.svg").to_vec(),
    )
}

/// Only clipboard colour samples use a literal background.
pub fn swatch(color: Color) -> theme::Container<'static> {
    theme::Container::custom(move |theme| cosmic::iced::widget::container::Style {
        background: Some(color.into()),
        border: Border {
            radius: theme.cosmic().corner_radii.radius_s.into(),
            ..Default::default()
        },
        ..Default::default()
    })
}

/// Keep the existing density choices while delegating interaction states to COSMIC.
pub fn button(selected: bool, radius: f32, card: bool) -> theme::Button {
    styled_button(selected, radius, card, None)
}
/// Type filters remain colored when inactive; selection strengthens their fill.
pub fn filter_button(kind: Option<Kind>, selected: bool, mode: TypeColors) -> theme::Button {
    let Some(kind) = kind.filter(|_| mode != TypeColors::Off) else {
        return button(selected, 8.0, false);
    };
    use widget::button::Catalog;
    let decorate = move |mut style: widget::button::Style, theme: &cosmic::Theme, focus: bool| {
        let tint = TypeTint::Clip(kind, None).color();
        let base: Color = theme.cosmic().bg_color().into();
        let weight = if selected {
            0.85
        } else if mode == TypeColors::Vivid {
            0.40
        } else {
            0.24
        };
        let mut bg = blend(base, tint, weight);
        bg.a = 1.0;
        let fg = readable(tint, bg);
        style.background = Some(bg.into());
        style.text_color = Some(fg);
        style.icon_color = Some(fg);
        if selected || focus {
            style.border_width = 2.0;
            style.border_color = readable(tint, bg);
        }
        style
    };
    theme::Button::Custom {
        active: Box::new(move |focus, theme| {
            decorate(
                theme.active(focus, false, &theme::Button::Standard),
                theme,
                focus,
            )
        }),
        hovered: Box::new(move |focus, theme| {
            decorate(
                theme.hovered(focus, false, &theme::Button::Standard),
                theme,
                focus,
            )
        }),
        pressed: Box::new(move |focus, theme| {
            decorate(
                theme.pressed(focus, false, &theme::Button::Standard),
                theme,
                focus,
            )
        }),
        disabled: Box::new(move |theme| theme.disabled(&theme::Button::Standard)),
    }
}
pub fn type_button(
    selected: bool,
    tint: TypeTint,
    mode: TypeColors,
    compact: bool,
) -> theme::Button {
    styled_button(selected, 8.0, true, Some((tint, mode, compact)))
}
fn styled_button(
    selected: bool,
    _radius: f32,
    card: bool,
    tint: Option<(TypeTint, TypeColors, bool)>,
) -> theme::Button {
    if !card {
        return if selected {
            theme::Button::Suggested
        } else {
            theme::Button::Text
        };
    }
    // Native interaction colours, but card-sized corners instead of a pill.
    use widget::button::Catalog;
    let decorate = move |mut style: widget::button::Style, theme: &cosmic::Theme, focus: bool| {
        if let Some((tint, mode, compact)) = tint {
            if let Some(cosmic::iced::Background::Color(bg)) = style.background {
                style.background = Some(blend(bg, tint.color(), strength(mode, compact)).into());
            }
        }
        style.border_radius = theme.cosmic().corner_radii.radius_s.into();
        if selected || focus {
            style.border_width = 2.0;
            style.border_color = theme.cosmic().accent_color().into();
        }
        style
    };
    theme::Button::Custom {
        active: Box::new(move |focus, theme| {
            decorate(
                theme.active(focus, false, &theme::Button::Standard),
                theme,
                focus,
            )
        }),
        hovered: Box::new(move |focus, theme| {
            decorate(
                theme.hovered(focus, false, &theme::Button::Standard),
                theme,
                focus,
            )
        }),
        pressed: Box::new(move |focus, theme| {
            decorate(
                theme.pressed(focus, false, &theme::Button::Standard),
                theme,
                focus,
            )
        }),
        disabled: Box::new(move |theme| {
            decorate(theme.disabled(&theme::Button::Standard), theme, false)
        }),
    }
}
pub fn input() -> theme::TextInput {
    theme::TextInput::Default
}

/// Tooltips name icon-only actions in both supported languages.
pub fn hint<'a, M: Clone + 'a>(
    content: impl Into<cosmic::Element<'a, M>>,
    label: &'a str,
) -> cosmic::Element<'a, M> {
    widget::tooltip(
        content,
        widget::text(label),
        widget::tooltip::Position::Bottom,
    )
    .into()
}

pub fn icon(name: &str) -> widget::icon::Handle {
    let path = match name {
        "edit-paste-symbolic" => {
            "<rect x='5' y='5' width='14' height='16' rx='2'/><rect x='9' y='3' width='6' height='4' rx='1'/><path d='M9 12h6m-6 4h6'/>"
        }
        "media-playback-start-symbolic" => "<path d='m8 4 12 8-12 8z'/>",
        "window-close-symbolic" => "<path d='m6 6 12 12M18 6 6 18'/>",
        "edit-delete-symbolic" => "<path d='M4 6h16M9 6V3h6v3M7 6l1 15h8l1-15M10 10v7m4-7v7'/>",
        "image-x-generic-symbolic" => {
            "<rect x='3' y='3' width='18' height='18' rx='2'/><circle cx='8' cy='8' r='1.5'/><path d='m3 17 5-5 4 4 4-6 5 7'/>"
        }
        "insert-link-symbolic" => {
            "<path d='m9 15 6-6m-7 3-2 2a4 4 0 0 0 6 6l2-2m-4-12 2-2a4 4 0 0 1 6 6l-2 2'/>"
        }
        "utilities-terminal-symbolic" => {
            "<rect x='3' y='4' width='18' height='16' rx='2'/><path d='m7 9 3 3-3 3m6 0h4'/>"
        }
        "applications-graphics-symbolic" => {
            "<circle cx='9' cy='9' r='5'/><circle cx='15' cy='9' r='5'/><circle cx='12' cy='15' r='5'/>"
        }
        "folder-symbolic" => "<path d='M3 6h7l2 3h9v11H3z'/>",
        "search" => "<circle cx='10' cy='10' r='6'/><path d='m15 15 6 6'/>",
        "emblem-system-symbolic" => {
            "<circle cx='12' cy='12' r='3'/><path d='M12 3v2m0 14v2M3 12h2m14 0h2M5.6 5.6l1.4 1.4m10 10 1.4 1.4m0-12.8-1.4 1.4m-10 10-1.4 1.4'/>"
        }
        "media-playback-pause-symbolic" => "<path d='M9 5v14M15 5v14'/>",
        "view-reveal-symbolic" => {
            "<path d='M2 12s3.6-6 10-6 10 6 10 6-3.6 6-10 6-10-6-10-6Z'/><circle cx='12' cy='12' r='2.5'/>"
        }
        "object-select-symbolic" => "<path d='m4 12 5 5L20 6'/>",
        _ => "<path d='M5 5h14M5 10h14M5 15h10M5 20h7'/>",
    };
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24'><g fill='none' stroke='#b4b7c1' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'>{path}</g></svg>"
    );
    widget::icon::from_svg_bytes(svg.into_bytes()).symbolic(true)
}

use nebula_paste::{
    model::{self, Clip, Kind},
    settings::TypeColors,
};
/// Kept separate from stored clip kinds: notes are not clipboard history.
#[derive(Clone, Copy)]
pub enum TypeTint {
    Clip(Kind, Option<[u8; 3]>),
    Note,
    Neutral,
}
impl TypeTint {
    pub fn clip(c: &Clip) -> Self {
        Self::Clip(
            c.kind,
            if c.kind == Kind::Color {
                model::color(c.text.trim())
            } else {
                None
            },
        )
    }
    fn color(self) -> Color {
        let rgb = match self {
            Self::Clip(Kind::Text, _) => [168, 143, 235],
            Self::Clip(Kind::Link, _) => [73, 151, 232],
            Self::Clip(Kind::Image, _) => [226, 118, 177],
            Self::Clip(Kind::Code, _) => [68, 180, 145],
            Self::Clip(Kind::Files, _) => [213, 151, 60],
            Self::Note => [202, 180, 66],
            Self::Clip(Kind::Color, sample) => sample.unwrap_or([168, 143, 235]),
            Self::Neutral => [144, 144, 144],
        };
        Color::from_rgb8(rgb[0], rgb[1], rgb[2])
    }
}
fn strength(mode: TypeColors, compact: bool) -> f32 {
    if compact {
        return 0.0;
    }
    match mode {
        TypeColors::Off => 0.0,
        TypeColors::Subtle => 0.04,
        TypeColors::Vivid => 0.18,
    }
}
fn blend(base: Color, tint: Color, weight: f32) -> Color {
    Color {
        r: base.r + (tint.r - base.r) * weight,
        g: base.g + (tint.g - base.g) * weight,
        b: base.b + (tint.b - base.b) * weight,
        a: base.a,
    }
}
fn luminance(c: Color) -> f32 {
    let linear = |v: f32| {
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
}
fn contrast(a: Color, b: Color) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
fn readable(color: Color, bg: Color) -> Color {
    let target = if contrast(Color::BLACK, bg) > contrast(Color::WHITE, bg) {
        Color::BLACK
    } else {
        Color::WHITE
    };
    for step in 0..=20 {
        let c = blend(color, target, step as f32 / 20.0);
        if contrast(c, bg) >= 4.5 {
            return c;
        }
    }
    target
}
pub fn type_card(tint: TypeTint, mode: TypeColors, compact: bool) -> theme::Container<'static> {
    theme::Container::custom(move |theme| {
        use cosmic::iced::widget::container::Catalog;
        let mut style = theme.style(&theme::Container::Card);
        if let Some(cosmic::iced::Background::Color(bg)) = style.background {
            style.background = Some(blend(bg, tint.color(), strength(mode, compact)).into());
        }
        style
    })
}
pub fn type_rail(tint: TypeTint, mode: TypeColors) -> theme::Container<'static> {
    if mode == TypeColors::Off {
        theme::Container::Transparent
    } else {
        swatch(tint.color())
    }
}
/// A small opaque badge gives its label a predictable, accessible contrast even
/// when the applet background is frosted. The desktop surface stays untouched.
pub fn type_badge<'a, M: 'a>(
    label: impl Into<String>,
    icon_name: &str,
    tint: TypeTint,
    mode: TypeColors,
) -> cosmic::Element<'a, M> {
    let row = widget::row([])
        .spacing(4)
        .push(icon(icon_name).icon().size(12))
        .push(widget::text(label.into()).size(11));
    widget::container(row)
        .padding([2, 4])
        .class(theme::Container::custom(move |theme| {
            if mode == TypeColors::Off {
                return Default::default();
            }
            let base: Color = theme.cosmic().bg_color().into();
            let mut bg = blend(base, tint.color(), 0.12);
            bg.a = 1.0;
            let fg = readable(tint.color(), bg);
            cosmic::iced::widget::container::Style {
                background: Some(bg.into()),
                text_color: Some(fg),
                icon_color: Some(fg),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into()
}

#[cfg(test)]
mod type_color_tests {
    use super::*;
    #[test]
    fn badge_labels_meet_contrast_on_both_themes_and_extreme_swatches() {
        for theme in [cosmic::Theme::dark(), cosmic::Theme::light()] {
            for tint in Kind::ALL
                .into_iter()
                .map(|k| TypeTint::Clip(k, None))
                .chain([
                    TypeTint::Note,
                    TypeTint::Clip(Kind::Color, Some([255; 3])),
                    TypeTint::Clip(Kind::Color, Some([0; 3])),
                ])
            {
                let bg = blend(theme.cosmic().bg_color().into(), tint.color(), 0.12);
                assert!(contrast(readable(tint.color(), bg), bg) >= 4.5);
            }
        }
    }
    #[test]
    fn off_keeps_native_card_and_selection_and_tints_keep_alpha() {
        use widget::button::Catalog;
        let theme = cosmic::Theme::dark();
        let tint = TypeTint::Clip(Kind::Image, None);
        for selected in [false, true] {
            let native = theme.active(false, false, &button(selected, 8.0, true));
            let off = theme.active(
                false,
                false,
                &type_button(selected, tint, TypeColors::Off, false),
            );
            assert_eq!(native.background, off.background);
            assert_eq!(native.border_color, off.border_color);
            let vivid = theme.active(
                false,
                false,
                &type_button(selected, tint, TypeColors::Vivid, false),
            );
            assert_eq!(vivid.border_color, native.border_color);
        }
        assert_eq!(
            blend(
                Color {
                    a: 0.7,
                    ..Color::BLACK
                },
                Color::WHITE,
                0.18
            )
            .a,
            0.7
        );
    }
}
