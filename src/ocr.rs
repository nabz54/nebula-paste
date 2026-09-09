//! Statically linked Tesseract C API. No external OCR process or system tessdata.
use nebula_paste::model::{self, Clip, Kind, MAX_CLIP_BYTES};
use std::{
    ffi::{CStr, CString, c_char, c_int, c_uchar, c_void},
    ptr::NonNull,
};

unsafe extern "C" {
    fn TessBaseAPICreate() -> *mut c_void;
    fn TessBaseAPIDelete(handle: *mut c_void);
    fn TessBaseAPIInit2(
        handle: *mut c_void,
        path: *const c_char,
        language: *const c_char,
        mode: c_int,
    ) -> c_int;
    fn TessBaseAPISetPageSegMode(handle: *mut c_void, mode: c_int);
    fn TessBaseAPISetImage(
        handle: *mut c_void,
        data: *const c_uchar,
        width: c_int,
        height: c_int,
        channels: c_int,
        stride: c_int,
    );
    fn TessBaseAPISetSourceResolution(handle: *mut c_void, ppi: c_int);
    fn TessBaseAPIRecognize(handle: *mut c_void, monitor: *mut c_void) -> c_int;
    fn TessBaseAPIGetUTF8Text(handle: *mut c_void) -> *mut c_char;
    fn TessDeleteText(text: *const c_char);
    fn TessMonitorCreate() -> *mut c_void;
    fn TessMonitorDelete(monitor: *mut c_void);
    fn TessMonitorSetDeadlineMSecs(monitor: *mut c_void, deadline: c_int);
}
struct Engine(NonNull<c_void>);
impl Drop for Engine {
    fn drop(&mut self) {
        // SAFETY: this wrapper exclusively owns a handle from TessBaseAPICreate.
        unsafe { TessBaseAPIDelete(self.0.as_ptr()) };
    }
}
struct Monitor(NonNull<c_void>);
impl Drop for Monitor {
    fn drop(&mut self) {
        // SAFETY: this wrapper exclusively owns a monitor from TessMonitorCreate.
        unsafe { TessMonitorDelete(self.0.as_ptr()) };
    }
}
struct Text(NonNull<c_char>);
impl Drop for Text {
    fn drop(&mut self) {
        // SAFETY: GetUTF8Text transfers this allocation; TessDeleteText releases it.
        unsafe { TessDeleteText(self.0.as_ptr()) };
    }
}

pub fn recognize(clip: Clip, language: &str) -> Result<String, String> {
    if clip.kind != Kind::Image {
        return Err("Sélectionne une image pour l’OCR.".into());
    }
    if !matches!(language, "fra" | "eng" | "fra+eng") {
        return Err("Langue OCR non prise en charge.".into());
    }
    let rgba = model::decode_image(&clip.bytes)?.into_rgba8();
    let (width, height) = rgba.dimensions();
    // Composite transparent pixels onto white, preserving dark text on transparent images.
    let mut rgb = Vec::with_capacity((width as usize) * (height as usize) * 3);
    for pixel in rgba.pixels() {
        let alpha = u16::from(pixel[3]);
        for &channel in &pixel.0[..3] {
            rgb.push(((u16::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255) as u8);
        }
    }
    drop(rgba);
    // Tesseract loads files by language name. Only bundled public model data is
    // staged here; the user's image and recognized text stay in memory.
    // TempDir is private (0700) and cleaned up on every normal exit/error path.
    let models = tempfile::tempdir().map_err(|e| e.to_string())?;
    for (name, bytes) in [
        (
            "eng",
            include_bytes!("../vendor/tessdata/eng.traineddata").as_slice(),
        ),
        (
            "fra",
            include_bytes!("../vendor/tessdata/fra.traineddata").as_slice(),
        ),
    ] {
        if language.split('+').any(|part| part == name) {
            std::fs::write(models.path().join(format!("{name}.traineddata")), bytes)
                .map_err(|e| e.to_string())?;
        }
    }
    use std::os::unix::ffi::OsStrExt;
    let path = CString::new(models.path().as_os_str().as_bytes()).map_err(|e| e.to_string())?;
    let language = CString::new(language).map_err(|e| e.to_string())?;
    // SAFETY: constructors return opaque handles checked for null, exclusively
    // owned by RAII guards. Strings remain valid for Init2; RGB storage remains
    // alive until recognition finishes. Dimensions/stride describe this exact
    // buffer and are bounded by model::decode_image's 8192-pixel dimension cap.
    unsafe {
        let engine =
            Engine(NonNull::new(TessBaseAPICreate()).ok_or("Initialisation OCR impossible.")?);
        if TessBaseAPIInit2(engine.0.as_ptr(), path.as_ptr(), language.as_ptr(), 1) != 0 {
            return Err("Impossible de charger les modèles OCR embarqués.".into());
        }
        TessBaseAPISetPageSegMode(engine.0.as_ptr(), 11);
        TessBaseAPISetImage(
            engine.0.as_ptr(),
            rgb.as_ptr(),
            width as c_int,
            height as c_int,
            3,
            (width * 3) as c_int,
        );
        TessBaseAPISetSourceResolution(engine.0.as_ptr(), 150);
        let monitor = Monitor(
            NonNull::new(TessMonitorCreate()).ok_or("Initialisation du suivi OCR impossible.")?,
        );
        TessMonitorSetDeadlineMSecs(monitor.0.as_ptr(), 30_000);
        if TessBaseAPIRecognize(engine.0.as_ptr(), monitor.0.as_ptr()) != 0 {
            return Err("Reconnaissance interrompue ou délai OCR dépassé.".into());
        }
        let result = Text(
            NonNull::new(TessBaseAPIGetUTF8Text(engine.0.as_ptr()))
                .ok_or("Aucun texte détecté.")?,
        );
        // Tesseract guarantees a NUL-terminated allocation on success.
        let bytes = CStr::from_ptr(result.0.as_ptr()).to_bytes();
        if bytes.len() > MAX_CLIP_BYTES {
            return Err("Texte OCR trop volumineux.".into());
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|e| e.to_string())?
            .to_owned();
        if text.trim().is_empty() {
            return Err("Aucun texte détecté dans cette image.".into());
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_french_and_combined_models_recognize_accents_on_transparency() {
        let clip = Clip::new(
            "image/png".into(),
            include_bytes!("../tests/fixtures/ocr-french.png").to_vec(),
            1,
        )
        .unwrap();
        for language in ["fra", "fra+eng"] {
            let result = recognize(clip.clone(), language).unwrap();
            for word in ["Été", "déjà", "élève", "café"] {
                assert!(result.contains(word), "{language}: {result}");
            }
        }
    }
    #[test]
    fn unsupported_language_is_rejected_without_loading_models() {
        let clip = Clip::new(
            "image/png".into(),
            include_bytes!("../tests/fixtures/ocr.png").to_vec(),
            1,
        )
        .unwrap();
        assert!(
            recognize(clip, "../../invalid")
                .unwrap_err()
                .contains("Langue")
        );
    }
    #[test]
    fn blank_image_returns_a_visible_error() {
        let image = image::RgbImage::from_pixel(200, 100, image::Rgb([255, 255, 255]));
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(image)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let clip = Clip::new("image/png".into(), bytes.into_inner(), 1).unwrap();
        assert!(recognize(clip, "eng").unwrap_err().contains("Aucun texte"));
    }
}
