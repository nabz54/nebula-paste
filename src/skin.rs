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
pub fn button(selected: bool, _radius: f32, card: bool) -> theme::Button {
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
