//! Local actions: embedded OCR and optional wtype. No shell interpolation.
use nebula_paste::model::Clip;
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn wait(command: &mut Command, timeout: Duration) -> Result<(), String> {
    let mut child = command
        .spawn()
        .map_err(|e| format!("Impossible de lancer l’outil local : {e}"))?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "L’outil local a échoué ({status}). Vérifie son installation et les autorisations du bureau."
                    ))
                };
            }
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(match result {
                    Err(e) => e.to_string(),
                    _ => "Délai dépassé ; opération annulée.".into(),
                });
            }
        }
    }
}

pub fn ocr(clip: Clip, language: &'static str) -> Result<String, String> {
    crate::ocr::recognize(clip, language)
}

pub fn paste(terminal: bool) -> Result<(), String> {
    let mut command = Command::new("wtype");
    command.args(["-M", "ctrl"]);
    if terminal {
        command.args(["-M", "shift"]);
    }
    command.args(["-P", "v", "-p", "v"]);
    if terminal {
        command.args(["-m", "shift"]);
    }
    command
        .args(["-m", "ctrl"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    wait(&mut command, Duration::from_secs(3)).map_err(|e| {
        format!("Collage direct : {e} Installe wtype. Le contenu reste copié : utilise Ctrl+V.")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_deadline_kills_and_reaps_child() {
        let start = Instant::now();
        assert!(
            wait(
                Command::new("/bin/sleep").arg("10"),
                Duration::from_millis(40)
            )
            .is_err()
        );
        assert!(start.elapsed() < Duration::from_secs(2));
    }
    #[test]
    fn ocr_rejects_non_images_before_loading_engine() {
        let clip = Clip::new("text/plain".into(), b"test".to_vec(), 1).unwrap();
        assert!(ocr(clip, "eng").unwrap_err().contains("image"));
    }
    #[test]
    fn recognizes_real_image_locally() {
        let clip = Clip::new(
            "image/png".into(),
            include_bytes!("../tests/fixtures/ocr.png").to_vec(),
            1,
        )
        .unwrap();
        let text = ocr(clip, "eng").unwrap();
        assert!(text.contains("NEBULA PASTE 12345"), "{text}");
    }
}
