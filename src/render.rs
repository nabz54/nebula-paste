//! Render the actual widget tree to PNG, without a window server or clipboard.
use cosmic::{
    Application,
    iced::{
        self, Rectangle, Size,
        advanced::{
            Layout,
            renderer::{Headless, Renderer as _},
            widget::Tree,
        },
        mouse,
    },
};
pub fn preview(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (mut app, _) = crate::app::App::init(cosmic::Core::default(), crate::app::Mode::Preview);
    if std::env::args().nth(3).as_deref() == Some("detail") {
        let id = crate::demo::clips()[0].id.clone();
        let _ = app.update(crate::app::Message::Detail(Some(id)));
    }
    if std::env::args().nth(3).as_deref() == Some("full") {
        app.expand_demo();
    }
    if matches!(
        std::env::args().nth(3).as_deref(),
        Some("list" | "settings")
    ) {
        app.expand_demo();
        let _ = app.update(crate::app::Message::Density);
    }
    if std::env::args().nth(3).as_deref() == Some("settings") {
        let _ = app.update(crate::app::Message::Settings(true));
    }
    if std::env::args().nth(3).as_deref() == Some("narrow") {
        app.expand_demo();
        let _ = app.update(crate::app::Message::Viewport(360.0));
    }
    if std::env::args().nth(3).as_deref() == Some("collections") {
        let _ = app.update(crate::app::Message::Collections(true));
    }
    let mode = std::env::args().nth(3).unwrap_or_default();
    if mode.starts_with("media") {
        let _ = app.update(crate::app::Message::MediaOpen);
        if !mode.contains("home") {
            let clip = if mode.contains("color") {
                nebula_paste::imaging::color_clip([0.55, 0.4, 0.85])?
            } else {
                crate::demo::clips()
                    .into_iter()
                    .find(|c| c.kind == nebula_paste::model::Kind::Image)
                    .unwrap()
            };
            let prepared = crate::media_ui::prepare(std::sync::Arc::new(clip))?;
            let _ = app.update(crate::app::Message::Media(
                crate::media_ui::Message::Loaded(Ok(Some(prepared)), false),
            ));
            if mode.contains("ocr") {
                let _ = app.update(crate::app::Message::Media(
                    crate::media_ui::Message::OcrDone(Ok(
                        "Texte reconnu / Recognized text\nVérifie le résultat avant de copier."
                            .into(),
                    )),
                ));
            }
        }
    }
    if mode.starts_with("actions") {
        app.expand_demo();
        let parts = crate::demo::clips()
            .iter()
            .filter_map(crate::action_ui::clip_part)
            .take(3)
            .collect();
        let _ = app.update(crate::app::Message::ActionsLoad(parts));
        let _ = app.update(crate::app::Message::Action(
            crate::action_ui::Message::MakeQueue,
        ));
    }
    if mode.starts_with("workspace") {
        app.expand_demo();
        let _ = app.update(crate::app::Message::Workspace(true));
        if mode.contains("list") {
            let _ = app.update(crate::app::Message::WorkspaceEvent(
                crate::workspace_ui::Message::Layout,
            ));
        }
    }
    if mode.starts_with("notes") {
        app.expand_demo();
        let _ = app.update(crate::app::Message::Notes(true));
        if mode.contains("edit") {
            let _ = app.update(crate::app::Message::Note(crate::note_ui::Message::New));
            let _ = app.update(crate::app::Message::Note(crate::note_ui::Message::Title(
                "Idées / Ideas".into(),
            )));
        }
    }
    if mode.starts_with("templates") {
        let _ = app.update(crate::app::Message::Templates(true));
        if mode.contains("edit") {
            let _ = app.update(crate::app::Message::Template(
                crate::template_ui::Message::New,
            ));
            let _ = app.update(crate::app::Message::Template(
                crate::template_ui::Message::Title("Réponse de suivi / Follow-up".into()),
            ));
        }
    }
    if mode.starts_with("data") {
        app.prepare_data_preview();
    }
    // Fifth argument selects the presentation setting without touching user data.
    match std::env::args().nth(5).as_deref() {
        Some("vivid") => {
            let _ = app.update(crate::app::Message::TypeColors);
        }
        Some("off") => {
            let _ = app.update(crate::app::Message::TypeColors);
            let _ = app.update(crate::app::Message::TypeColors);
        }
        _ => {}
    }
    let popup = mode == "ribbon" || mode == "popup" || mode.contains("popup");
    if popup {
        app.render_popup();
    }
    if mode == "ribbon" {
        app.render_ribbon();
    }
    let theme = if std::env::args().nth(4).as_deref() == Some("light") {
        cosmic::Theme::light()
    } else {
        cosmic::Theme::dark()
    };
    app.render_theme(theme.clone());
    let mut view = if mode.starts_with("data") {
        app.render_data_panel(popup)
    } else if popup {
        app.view_window(iced::window::Id::unique())
    } else {
        app.view()
    };
    let mut renderer = iced::futures::executor::block_on(<cosmic::Renderer as Headless>::new(
        iced::Font::DEFAULT,
        iced::Pixels(14.0),
        Some("tiny-skia"),
    ))
    .ok_or("Renderer unavailable")?;
    let mut tree = Tree::new(&view);
    let limits = iced::advanced::layout::Limits::new(Size::ZERO, Size::new(940.0, 850.0));
    let node = view.as_widget_mut().layout(&mut tree, &renderer, &limits);
    let size = node.size();
    let bounds = Rectangle::with_size(size);
    renderer.reset(bounds);
    view.as_widget().draw(
        &tree,
        &mut renderer,
        &theme,
        &iced::advanced::renderer::Style {
            text_color: theme.cosmic().on_bg_color().into(),
            icon_color: theme.cosmic().on_bg_color().into(),
            scale_factor: 1.0,
        },
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &bounds,
    );
    let pixels = renderer.screenshot(
        Size::new(size.width.ceil() as u32, size.height.ceil() as u32),
        1.0,
        theme.cosmic().bg_color().into(),
    );
    image::save_buffer(
        path,
        &pixels,
        size.width.ceil() as u32,
        size.height.ceil() as u32,
        image::ColorType::Rgba8,
    )?;
    println!("Rendered {} × {} to {path}", size.width, size.height);
    Ok(())
}
