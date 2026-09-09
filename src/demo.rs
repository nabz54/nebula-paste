//! Isolated, in-memory fixture data for visual inspection.
use nebula_paste::model::{Clip, now};
pub fn clips() -> Vec<Clip> {
    let mut entries: Vec<Clip> =
    ["sudo systemctl status httpd\n\n# Vérifier le service Apache\nsudo journalctl -u httpd -n 50", "#AFA3FF", "https://system76.com/cosmic", "Bonjour,\n\nVoici les éléments pour notre prochain échange.\nBonne journée !"].iter().enumerate().map(|(i,text)| {
        let mut c=Clip::new("text/plain;charset=utf-8".into(),text.as_bytes().to_vec(),now()-i as i64*120).unwrap();
        c.pinned=i==0;c.category=match i { 0=>"Commandes".into(),3=>"Modèles".into(),_=>String::new() };c
    }).collect();
    let image = image::RgbImage::from_fn(320, 180, |x, y| {
        image::Rgb([
            (50 + x * 120 / 320) as u8,
            (75 + y * 80 / 180) as u8,
            (200 - x * 30 / 320) as u8,
        ])
    });
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(image)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let picture = Clip::new("image/png".into(), png.into_inner(), now() + 1).unwrap();
    entries.insert(0, picture);
    entries.truncate(4);
    entries
}
