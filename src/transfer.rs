use cosmic::iced::clipboard::mime::AsMimeTypes;
use nebula_paste::model::{Clip, Kind};
use std::{borrow::Cow, sync::Arc};

pub const CLIP_ID_MIME: &str = "application/x-nebula-paste-clip-id";

/// Share payload storage between redraws and drag requests.
#[derive(Clone)]
pub struct Payload {
    pub mime: String,
    pub id: String,
    pub bytes: Arc<[u8]>,
    pub text: bool,
}
impl Payload {
    pub fn from_clip(clip: &Clip) -> Self {
        Self {
            mime: clip.mime.clone(),
            id: clip.id.clone(),
            bytes: Arc::from(clip.bytes.as_slice()),
            text: !matches!(clip.kind, Kind::Image | Kind::Files),
        }
    }
}
impl AsMimeTypes for Payload {
    fn available(&self) -> Cow<'static, [String]> {
        let mut mimes = vec![self.mime.clone(), CLIP_ID_MIME.into()];
        if self.text {
            for mime in ["text/plain;charset=utf-8", "text/plain"] {
                if !mimes.iter().any(|m| m == mime) {
                    mimes.push(mime.into());
                }
            }
        }
        Cow::Owned(mimes)
    }
    fn as_bytes(&self, mime: &str) -> Option<Cow<'static, [u8]>> {
        if mime == CLIP_ID_MIME {
            return Some(Cow::Owned(self.id.as_bytes().to_vec()));
        }
        self.available()
            .iter()
            .any(|m| m == mime)
            .then(|| Cow::Owned(self.bytes.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_aliases_preserve_original_bytes() {
        let clip = Clip::new(
            "text/plain;charset=utf-8".into(),
            "  été\n".as_bytes().to_vec(),
            1,
        )
        .unwrap();
        let payload = Payload::from_clip(&clip);
        assert_eq!(payload.as_bytes("text/plain").unwrap().as_ref(), clip.bytes);
        assert_eq!(payload.available().len(), 3);
        assert_eq!(
            payload.as_bytes(CLIP_ID_MIME).unwrap().as_ref(),
            clip.id.as_bytes()
        );
        assert!(payload.as_bytes("image/png").is_none());
    }
    #[test]
    fn files_are_offered_as_uris_without_fake_text_or_move() {
        let clip = Clip::new(
            "text/uri-list".into(),
            b"file:///tmp/test%20file.txt\r\n".to_vec(),
            1,
        )
        .unwrap();
        let payload = Payload::from_clip(&clip);
        assert_eq!(
            payload.available().as_ref(),
            ["text/uri-list", CLIP_ID_MIME]
        );
        assert!(payload.as_bytes("text/plain").is_none());
    }
}
