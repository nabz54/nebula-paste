use image::{DynamicImage, Rgba, RgbaImage};
use nebula_paste::{
    imaging::{self, Format, Plan},
    model,
};
fn sample() -> Vec<u8> {
    imaging::encode(
        &DynamicImage::ImageRgba8(RgbaImage::from_fn(4, 3, |x, y| {
            Rgba([x as u8 * 60, y as u8 * 80, 20, 128])
        })),
        Format::Png,
        85,
    )
    .unwrap()
    .bytes
}
#[test]
fn crop_then_resize_preserves_source_and_png_alpha() {
    let original = sample();
    let before = original.clone();
    let out = imaging::transform(
        &original,
        Plan {
            crop: [1, 1, 2, 2],
            size: [2, 2],
            format: Format::Png,
            quality: 85,
        },
    )
    .unwrap();
    let img = model::decode_image(&out.bytes).unwrap().into_rgba8();
    assert_eq!(img.dimensions(), (2, 2));
    assert_eq!(img.get_pixel(0, 0).0, [60, 80, 20, 128]);
    assert_eq!(original, before);
    let resized = imaging::transform(
        &original,
        Plan {
            crop: [0, 0, 4, 3],
            size: [8, 6],
            format: Format::Png,
            quality: 85,
        },
    )
    .unwrap();
    assert_eq!(model::decode_image(&resized.bytes).unwrap().width(), 8);
}
#[test]
fn formats_round_trip_and_jpeg_flattens_transparency() {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(8, 8, Rgba([0, 0, 0, 0])));
    for format in Format::ALL {
        let c = imaging::encode(&image, format, 95).unwrap();
        assert_eq!(c.mime, format.mime());
        let decoded = model::decode_image(&c.bytes).unwrap().into_rgba8();
        if format == Format::Jpeg {
            assert!(decoded.get_pixel(0, 0).0[..3].iter().all(|n| *n > 248));
        } else {
            assert_eq!(decoded.get_pixel(0, 0)[3], 0);
        }
    }
}
#[test]
fn invalid_crops_and_memory_budgets_are_rejected() {
    for crop in [[0, 0, 0, 1], [4, 0, 1, 1], [u32::MAX, 0, 2, 2]] {
        assert!(
            Plan {
                crop,
                size: [2, 2],
                format: Format::Png,
                quality: 85
            }
            .validate([4, 3])
            .is_err()
        );
    }
    for size in [[0, 1], [8193, 2], [8192, 8192]] {
        assert!(
            Plan {
                crop: [0, 0, 4, 3],
                size,
                format: Format::Png,
                quality: 85
            }
            .validate([4, 3])
            .is_err()
        );
    }
    assert!(
        imaging::transform(
            b"not an image",
            Plan {
                crop: [0, 0, 4, 3],
                size: [2, 2],
                format: Format::Png,
                quality: 85
            }
        )
        .is_err()
    );
}
#[test]
fn portal_colors_are_validated_and_rounded() {
    assert_eq!(
        imaging::color_clip([1.0, 0.5, 0.0]).unwrap().text,
        "#FF8000"
    );
    for value in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        assert!(imaging::color_clip([value, 0.0, 0.0]).is_err());
    }
}
