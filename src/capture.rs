//! Native desktop portal and explicit local-file import. No shell helpers.
use nebula_paste::{
    imaging,
    model::{self, Clip, MAX_CLIP_BYTES},
    tr,
};
use std::{io::Read, path::Path, sync::Arc, time::Duration};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    Image,
    Text,
    Color,
}
fn error(e: ashpd::Error) -> Result<Option<Arc<Clip>>, String> {
    if matches!(
        e,
        ashpd::Error::Response(ashpd::desktop::ResponseError::Cancelled)
            | ashpd::Error::Portal(ashpd::PortalError::Cancelled(_))
    ) {
        return Ok(None);
    }
    Err(format!(
        "{}: {e}",
        tr!(
            "Dialogue du bureau indisponible. Tu peux importer une image dans l’atelier.",
            "Desktop dialog unavailable. You can import an image into the workbench."
        )
    ))
}
pub fn read_image(path: &Path) -> Result<Arc<Clip>, String> {
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let metadata = f.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_CLIP_BYTES as u64 {
        return Err(tr!(
            "Fichier image local limité à 16 Mio.",
            "Local image file limited to 16 MiB."
        )
        .into());
    }
    let mut bytes = Vec::new();
    f.take(MAX_CLIP_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_CLIP_BYTES {
        return Err("Image exceeds 16 MiB".into());
    }
    let mime = match image::guess_format(&bytes).map_err(|e| e.to_string())? {
        image::ImageFormat::Png => "image/png",
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::WebP => "image/webp",
        _ => return Err("PNG / JPEG / WebP required".into()),
    };
    Clip::new(mime.into(), bytes, model::now()).map(Arc::new)
}
pub async fn portal(kind: Capture) -> Result<Option<Arc<Clip>>, String> {
    let operation = async {
        if kind == Capture::Color {
            let request = match ashpd::desktop::Color::pick().send().await {
                Ok(r) => r,
                Err(e) => return error(e),
            };
            let c = match request.response() {
                Ok(c) => c,
                Err(e) => return error(e),
            };
            imaging::color_clip([c.red(), c.green(), c.blue()]).map(|c| Some(Arc::new(c)))
        } else {
            let request = match ashpd::desktop::screenshot::Screenshot::request()
                .interactive(true)
                .modal(false)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => return error(e),
            };
            let screenshot = match request.response() {
                Ok(c) => c,
                Err(e) => return error(e),
            };
            let path = screenshot.uri().to_file_path().map_err(|_| {
                tr!(
                    "Le bureau n’a pas fourni de fichier local.",
                    "Desktop did not return a local file."
                )
                .to_string()
            })?;
            // Portal owns its output file; never remove an arbitrary returned path.
            tokio::task::spawn_blocking(move || read_image(&path).map(Some))
                .await
                .map_err(|e| e.to_string())?
        }
    };
    tokio::time::timeout(Duration::from_secs(180), operation)
        .await
        .map_err(|_| {
            tr!(
                "Le dialogue du bureau n’a pas répondu. Ferme-le avant de réessayer.",
                "Desktop dialog timed out. Close it before trying again."
            )
            .to_string()
        })?
}
pub async fn import() -> Result<Option<Arc<Clip>>, String> {
    use cosmic::dialog::file_chooser;
    let r = match file_chooser::open::Dialog::new()
        .title(tr!("Importer une image", "Import an image"))
        .filter(
            file_chooser::FileFilter::new("PNG / JPEG / WebP")
                .glob("*.png")
                .glob("*.jpg")
                .glob("*.jpeg")
                .glob("*.webp"),
        )
        .open_file()
        .await
    {
        Ok(r) => r,
        Err(file_chooser::Error::Cancelled) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let path = r.url().to_file_path().map_err(|_| "Local file required")?;
    tokio::task::spawn_blocking(move || read_image(&path).map(Some))
        .await
        .map_err(|e| e.to_string())?
}
pub async fn export(clip: Arc<Clip>) -> Result<bool, String> {
    use cosmic::dialog::file_chooser;
    let ext = match clip.mime.as_str() {
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => "png",
    };
    let name = format!("nebula-image-{}.{}", model::now(), ext);
    let r = match file_chooser::save::Dialog::new()
        .title(tr!("Exporter une nouvelle image", "Export a new image").into())
        .file_name(name)
        .save_file()
        .await
    {
        Ok(r) => r,
        Err(file_chooser::Error::Cancelled) => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    let Some(url) = r.url() else { return Ok(false) };
    let path = url.to_file_path().map_err(|_| "Local file required")?;
    if !path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
        s.eq_ignore_ascii_case(ext) || (ext == "jpg" && s.eq_ignore_ascii_case("jpeg"))
    }) {
        return Err(format!(
            "{} .{ext}",
            tr!("Choisis l’extension", "Use the extension")
        ));
    }
    tokio::task::spawn_blocking(move || nebula_paste::backup::write_new(&path, &clip.bytes))
        .await
        .map_err(|e| e.to_string())??;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_is_not_a_failure() {
        assert!(
            error(ashpd::Error::Response(
                ashpd::desktop::ResponseError::Cancelled
            ))
            .unwrap()
            .is_none()
        );
    }
    #[test]
    fn import_rejects_non_images_and_oversized_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.png");
        std::fs::write(&path, b"not png").unwrap();
        assert!(read_image(&path).is_err());
        std::fs::File::create(&path)
            .unwrap()
            .set_len(MAX_CLIP_BYTES as u64 + 1)
            .unwrap();
        assert!(read_image(&path).is_err());
    }
}
