//! Bounded image operations. Never mutate or replace the original clip.
use crate::{
    model::{self, Clip, MAX_CLIP_BYTES},
    tr,
};
use image::{DynamicImage, GenericImageView, ImageEncoder};
use std::io::{self, Write};

pub const MAX_PIXELS: u64 = 16_777_216;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Format {
    #[default]
    Png,
    Jpeg,
    Webp,
}
impl Format {
    pub const ALL: [Self; 3] = [Self::Png, Self::Jpeg, Self::Webp];
    pub fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Webp => "WebP",
        }
    }
    pub fn mime(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Plan {
    pub crop: [u32; 4],
    pub size: [u32; 2],
    pub format: Format,
    pub quality: u8,
}
impl Plan {
    pub fn validate(self, source: [u32; 2]) -> Result<(), String> {
        let [x, y, w, h] = self.crop;
        let [ow, oh] = self.size;
        if w == 0
            || h == 0
            || x.checked_add(w).is_none_or(|n| n > source[0])
            || y.checked_add(h).is_none_or(|n| n > source[1])
        {
            return Err(tr!(
                "Le recadrage dépasse l’image ou est vide.",
                "Crop is empty or outside the image."
            )
            .into());
        }
        if ow == 0
            || oh == 0
            || ow > 8192
            || oh > 8192
            || u64::from(ow) * u64::from(oh) > MAX_PIXELS
        {
            return Err(tr!(
                "Dimensions : 1–8192 pixels et 16 mégapixels maximum.",
                "Dimensions: 1–8192 pixels and at most 16 megapixels."
            )
            .into());
        }
        if !(1..=100).contains(&self.quality) {
            return Err("JPEG quality: 1–100".into());
        }
        Ok(())
    }
}
struct Limited(Vec<u8>);
impl Write for Limited {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_CLIP_BYTES.saturating_sub(self.0.len()) {
            return Err(io::Error::other("Image exceeds 16 MiB"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub fn transform(bytes: &[u8], plan: Plan) -> Result<Clip, String> {
    if bytes.len() > MAX_CLIP_BYTES {
        return Err("Image exceeds 16 MiB".into());
    }
    let image = model::decode_image(bytes)?;
    let (w, h) = image.dimensions();
    plan.validate([w, h])?;
    let [x, y, cw, ch] = plan.crop;
    let cropped = image.crop_imm(x, y, cw, ch);
    drop(image);
    let resized = if [cw, ch] == plan.size {
        cropped
    } else {
        cropped.resize_exact(
            plan.size[0],
            plan.size[1],
            image::imageops::FilterType::Lanczos3,
        )
    };
    encode(&resized, plan.format, plan.quality)
}
pub fn encode(image: &DynamicImage, format: Format, quality: u8) -> Result<Clip, String> {
    let (w, h) = image.dimensions();
    Plan {
        crop: [0, 0, w, h],
        size: [w, h],
        format,
        quality,
    }
    .validate([w, h])?;
    let mut out = Limited(Vec::new());
    match format {
        Format::Png => image::codecs::png::PngEncoder::new(&mut out).write_image(
            image.to_rgba8().as_raw(),
            w,
            h,
            image::ExtendedColorType::Rgba8,
        ),
        Format::Webp => image::codecs::webp::WebPEncoder::new_lossless(&mut out).write_image(
            image.to_rgba8().as_raw(),
            w,
            h,
            image::ExtendedColorType::Rgba8,
        ),
        Format::Jpeg => {
            let rgba = image.to_rgba8();
            let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
            for p in rgba.pixels() {
                let a = u16::from(p[3]);
                for &c in &p.0[..3] {
                    rgb.push(((u16::from(c) * a + 255 * (255 - a) + 127) / 255) as u8);
                }
            }
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality).encode(
                &rgb,
                w,
                h,
                image::ExtendedColorType::Rgb8,
            )
        }
    }
    .map_err(|e| e.to_string())?;
    Clip::new(format.mime().into(), out.0, model::now())
}
/// Reject non-finite / invalid portal RGB values instead of silently casting.
pub fn color_clip(rgb: [f64; 3]) -> Result<Clip, String> {
    if rgb
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return Err("Invalid color from desktop portal".into());
    }
    let [r, g, b] = rgb.map(|v| (v * 255.0).round() as u8);
    Clip::new(
        "text/plain;charset=utf-8".into(),
        format!("#{r:02X}{g:02X}{b:02X}").into_bytes(),
        model::now(),
    )
}
