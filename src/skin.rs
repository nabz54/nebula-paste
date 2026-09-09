//! Fixed opaque surfaces; never inherit the panel's transparency or pill radii.
use cosmic::{
    iced::{Border, Color},
    theme, widget,
};
// Nebula 0.5: quiet slate surfaces, cyan interaction, violet identity.
pub const BG: Color = Color::from_rgb(0.063, 0.078, 0.106);
pub const CARD: Color = Color::from_rgb(0.102, 0.125, 0.161);
pub const PREVIEW: Color = Color::from_rgb(0.078, 0.094, 0.125);
pub const LINE: Color = Color::from_rgb(0.208, 0.247, 0.302);
pub const TEXT: Color = Color::from_rgb(0.941, 0.953, 0.980);
pub const MUTED: Color = Color::from_rgb(0.671, 0.718, 0.788);
pub const ACCENT: Color = Color::from_rgb(0.451, 0.875, 0.827);
pub const SELECTED: Color = Color::from_rgb(0.102, 0.196, 0.212);
pub const HOVER: Color = Color::from_rgb(0.153, 0.192, 0.243);
pub fn brand_icon() -> widget::icon::Handle {
    widget::icon::from_svg_bytes(
        include_bytes!("../resources/io.github.nebulapaste.NebulaPaste.svg").to_vec(),
    )
}
pub fn surface(color: Color, radius: f32, border: Color) -> theme::Container<'static> {
    theme::Container::custom(move |_| cosmic::iced::widget::container::Style {
        background: Some(color.into()),
        text_color: Some(TEXT),
        icon_color: Some(TEXT),
        border: Border {
            radius: radius.into(),
            width: 1.0,
            color: border,
        },
        ..Default::default()
    })
}
pub fn button(selected: bool, radius: f32, card: bool) -> theme::Button {
    let style = move |hover: bool, focus: bool| widget::button::Style {
        background: Some(
            (if hover {
                HOVER
            } else if selected {
                SELECTED
            } else if card {
                CARD
            } else {
                BG
            })
            .into(),
        ),
        text_color: Some(if selected { TEXT } else { MUTED }),
        icon_color: Some(if selected { TEXT } else { MUTED }),
        border_radius: radius.into(),
        border_width: if selected || focus || card { 1.0 } else { 0.0 },
        border_color: if selected || focus { ACCENT } else { LINE },
        ..Default::default()
    };
    theme::Button::Custom {
        active: Box::new(move |focus, _| style(false, focus)),
        hovered: Box::new(move |focus, _| style(true, focus)),
        pressed: Box::new(move |focus, _| style(true, focus)),
        disabled: Box::new(move |_| {
            let mut s = style(false, false);
            s.text_color = Some(LINE);
            s.icon_color = Some(LINE);
            s
        }),
    }
}
pub fn input() -> theme::TextInput {
    let style = |focus| widget::text_input::Appearance {
        background: PREVIEW.into(),
        border_radius: 9.0.into(),
        border_width: 1.0,
        border_offset: None,
        border_color: if focus { ACCENT } else { LINE },
        text_color: Some(TEXT),
        icon_color: Some(MUTED),
        placeholder_color: MUTED,
        label_color: MUTED,
        selected_text_color: BG,
        selected_fill: ACCENT,
    };
    theme::TextInput::Custom {
        active: Box::new(move |_| style(false)),
        hovered: Box::new(move |_| style(true)),
        focused: Box::new(move |_| style(true)),
        error: Box::new(move |_| style(true)),
        disabled: Box::new(move |_| style(false)),
    }
}

pub fn icon(name: &str) -> widget::icon::Handle {
    let path = match name {
        "edit-paste-symbolic" => {
            "<rect x='5' y='5' width='14' height='16' rx='2'/><rect x='9' y='3' width='6' height='4' rx='1'/><path d='M9 12h6m-6 4h6'/>"
        }
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
        _ => "<path d='M5 5h14M5 10h14M5 15h10M5 20h7'/>",
    };
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24'><g fill='none' stroke='#b4b7c1' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'>{path}</g></svg>"
    );
    widget::icon::from_svg_bytes(svg.into_bytes())
}

pub fn kind_color(kind: nebula_paste::model::Kind) -> Color {
    use nebula_paste::model::Kind;
    match kind {
        Kind::Image => Color::from_rgb8(195, 166, 255),
        Kind::Code => Color::from_rgb8(107, 215, 177),
        Kind::Link => Color::from_rgb8(119, 185, 255),
        Kind::Color => Color::from_rgb8(242, 174, 214),
        Kind::Files => Color::from_rgb8(242, 201, 128),
        Kind::Text => Color::from_rgb8(184, 197, 220),
    }
}
