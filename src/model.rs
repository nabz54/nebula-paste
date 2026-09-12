use crate::{tr, tr_format};
use sha2::{Digest, Sha256};
use std::io::Cursor;

pub const MAX_CLIP_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_ITEMS: usize = 500;
pub const MAX_HISTORY_BYTES: usize = 128 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    Link,
    Image,
    Color,
    Files,
    Code,
}

impl Kind {
    pub const ALL: [Self; 6] = [
        Self::Text,
        Self::Link,
        Self::Image,
        Self::Color,
        Self::Files,
        Self::Code,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Text => tr!("Textes", "Text"),
            Self::Link => tr!("Liens", "Links"),
            Self::Image => tr!("Images", "Images"),
            Self::Color => tr!("Couleurs", "Colors"),
            Self::Files => tr!("Fichiers", "Files"),
            Self::Code => "Code",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Self::Text => "text-x-generic-symbolic",
            Self::Link => "insert-link-symbolic",
            Self::Image => "image-x-generic-symbolic",
            Self::Color => "applications-graphics-symbolic",
            Self::Files => "folder-symbolic",
            Self::Code => "utilities-terminal-symbolic",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Clip {
    pub id: String,
    pub mime: String,
    pub bytes: Vec<u8>,
    pub kind: Kind,
    pub title: String,
    pub text: String,
    pub timestamp: i64,
    pub pinned: bool,
    pub category: String,
}

impl Clip {
    pub fn new(mime: String, bytes: Vec<u8>, timestamp: i64) -> Result<Self, String> {
        if bytes.is_empty() || bytes.len() > MAX_CLIP_BYTES {
            return Err(tr!(
                "Contenu vide ou supérieur à 16 Mio",
                "Empty content or larger than 16 MiB"
            )
            .into());
        }
        let mut hash = Sha256::new();
        hash.update(mime.as_bytes());
        hash.update([0]);
        hash.update(&bytes);
        let id = format!("{:x}", hash.finalize());
        let (kind, title, text) = if mime.starts_with("image/") {
            let img = decode_image(&bytes)?;
            (
                Kind::Image,
                format!("Image · {} × {}", img.width(), img.height()),
                String::new(),
            )
        } else {
            let text = String::from_utf8(bytes.clone())
                .map_err(|_| tr!("Texte non UTF-8", "Text is not UTF-8"))?;
            let trimmed = text.trim();
            let kind = if mime == "text/uri-list" {
                Kind::Files
            } else if color(trimmed).is_some() {
                Kind::Color
            } else if (trimmed.starts_with("https://") || trimmed.starts_with("http://"))
                && !trimmed.contains(char::is_whitespace)
            {
                Kind::Link
            } else if [
                "fn ",
                "use ",
                "sudo ",
                "def ",
                "SELECT ",
                "#!/",
                "const ",
                "function ",
                "{\n",
            ]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
            {
                Kind::Code
            } else {
                Kind::Text
            };
            let title = trimmed
                .lines()
                .find(|line| !line.is_empty())
                .unwrap_or(tr!("Texte vide", "Empty text"))
                .chars()
                .take(64)
                .collect();
            (kind, title, text)
        };
        Ok(Self {
            id,
            mime,
            bytes,
            kind,
            title,
            text,
            timestamp,
            pinned: false,
            category: String::new(),
        })
    }
    pub fn matches(
        &self,
        query: &str,
        kind: Option<Kind>,
        favorites: bool,
        category: &str,
    ) -> bool {
        self.matches_with_ocr(query, kind, favorites, category, "")
    }
    pub fn matches_with_ocr(
        &self,
        query: &str,
        kind: Option<Kind>,
        favorites: bool,
        category: &str,
        ocr: &str,
    ) -> bool {
        (!favorites || self.pinned)
            && kind.is_none_or(|k| k == self.kind)
            && (category.is_empty() || self.category == category)
            && query.split_whitespace().all(|word| {
                let word = word.to_lowercase();
                self.text.to_lowercase().contains(&word)
                    || self.title.to_lowercase().contains(&word)
                    || self.category.to_lowercase().contains(&word)
                    || ocr.to_lowercase().contains(&word)
            })
    }
}

pub fn decode_image(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    reader
        .decode()
        .map_err(|e| format!("Image non prise en charge : {e}"))
}

pub fn color(value: &str) -> Option<[u8; 3]> {
    let hex = value.strip_prefix('#')?;
    if !hex.is_ascii() || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match hex.len() {
        3 => Some([
            u8::from_str_radix(&hex[0..1], 16).ok()? * 17,
            u8::from_str_radix(&hex[1..2], 16).ok()? * 17,
            u8::from_str_radix(&hex[2..3], 16).ok()? * 17,
        ]),
        6 => Some([
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        ]),
        _ => None,
    }
}

/// Plain text preserves text/code bytes. Only local file URIs become paths.
pub fn plain_text(clip: &Clip) -> Option<String> {
    if clip.kind == Kind::Image {
        return None;
    }
    if clip.mime != "text/uri-list" {
        return Some(clip.text.clone());
    }
    Some(
        clip.text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                if let Some(path) = line.strip_prefix("file:///") {
                    decode_percent(&format!("/{path}"))
                } else if let Some(path) = line.strip_prefix("file://localhost/") {
                    decode_percent(&format!("/{path}"))
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn decode_percent(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        // `get` refuse une tranche qui couperait un caractère multi-octets.
        if bytes[index] == b'%'
            && let Some(byte) = value
                .get(index + 1..index + 3)
                .filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()))
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        {
            out.push(byte);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| value.to_string())
}

/// Compte à rebours lisible pour la pause temporaire.
pub fn countdown(seconds: i64) -> String {
    let seconds = seconds.max(0);
    match seconds {
        0..60 => format!("{seconds} s"),
        60..3600 => format!("{} min {:02} s", seconds / 60, seconds % 60),
        _ => format!("{} h {:02} min", seconds / 3600, (seconds % 3600) / 60),
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
pub fn age(timestamp: i64) -> String {
    let seconds = (now() - timestamp).max(0);
    match seconds {
        0..60 => tr!("À l’instant", "Just now").into(),
        60..3600 => tr_format!("Il y a {} min", "{} min ago", seconds / 60),
        3600..86400 => tr_format!("Il y a {} h", "{} h ago", seconds / 3600),
        _ => tr_format!("Il y a {} j", "{} days ago", seconds / 86400),
    }
}
