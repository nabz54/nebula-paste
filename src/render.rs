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
    let popup = std::env::args().nth(3).as_deref() == Some("popup");
    if popup {
        app.render_popup();
    }
    let theme = if std::env::args().nth(4).as_deref() == Some("light") {
        cosmic::Theme::light()
    } else {
        cosmic::Theme::dark()
    };
    let mut view = if popup {
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
