mod actions;
mod app;
mod demo;
mod ipc;
mod ocr;
mod render;
mod skin;
mod transfer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nebula_paste::i18n::set_language("auto");
    match std::env::args().nth(1).as_deref() {
        Some("--ocr") => {
            use std::io::Read;
            let path = std::env::args()
                .nth(2)
                .ok_or("Indique une image PNG/JPEG")?;
            let language = std::env::args().nth(3).unwrap_or("fra+eng".into());
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take((nebula_paste::model::MAX_CLIP_BYTES + 1) as u64)
                .read_to_end(&mut bytes)?;
            let mime = match image::guess_format(&bytes)? {
                image::ImageFormat::Png => "image/png",
                image::ImageFormat::Jpeg => "image/jpeg",
                _ => return Err("Format PNG/JPEG requis".into()),
            };
            let clip = nebula_paste::model::Clip::new(mime.into(), bytes, 0)?;
            print!("{}", ocr::recognize(clip, &language)?);
            return Ok(());
        }
        Some("--render-preview") => {
            let path = std::env::args()
                .nth(2)
                .ok_or("Indique un fichier PNG de destination")?;
            return render::preview(&path);
        }
        Some("--preview") => {
            cosmic::app::run::<app::App>(
                cosmic::app::Settings::default()
                    .size(cosmic::iced::Size::new(940.0, 650.0))
                    .transparent(false)
                    .client_decorations(true),
                app::Mode::Preview,
            )?;
            return Ok(());
        }
        Some("--history") => {
            ipc::history()
                .map_err(|e| format!("Ajoute d’abord Nebula Paste au panneau COSMIC : {e}"))?;
            return Ok(());
        }
        Some("--toggle") => {
            ipc::toggle()
                .map_err(|e| format!("Ajoute d’abord Nebula Paste au panneau COSMIC : {e}"))?;
            return Ok(());
        }
        Some("--help") => {
            println!(
                "{}",
                nebula_paste::tr!(
                    "Nebula Paste\n  Sans option : lancer l’applet\n  --toggle : ouvrir/fermer l’applet déjà actif (raccourci COSMIC)\n  --history : ouvrir l’historique complet de l’applet\n  --preview : démonstration isolée\n  --render-preview fichier.png [full|detail|list|settings|narrow|popup] [light|dark] : capture de démonstration\n  --ocr image.png [fra|eng|fra+eng] : OCR embarqué\n  --version : version",
                    "Nebula Paste\n  No option: start the applet\n  --toggle: open/close the running applet\n  --history: open the applet’s full history window\n  --preview: isolated demo\n  --render-preview file.png [full|detail|list|settings|narrow|popup] [light|dark]: native widget screenshot\n  --ocr image.png [fra|eng|fra+eng]: embedded OCR\n  --version: version"
                )
            );
            return Ok(());
        }
        Some("--version") => {
            println!("Nebula Paste {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(arg) => return Err(format!("Option inconnue : {arg}").into()),
        None => {}
    }
    let instance = ipc::Instance::acquire()?;
    cosmic::applet::run::<app::App>(app::Mode::Applet(instance))?;
    Ok(())
}
