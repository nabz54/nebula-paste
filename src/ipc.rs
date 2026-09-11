use std::{
    fs::{File, OpenOptions},
    io,
    os::unix::{
        fs::{OpenOptionsExt, PermissionsExt},
        net::UnixDatagram,
    },
    path::PathBuf,
};

fn directory() -> io::Result<PathBuf> {
    let dir = dirs::runtime_dir()
        .ok_or_else(|| io::Error::other("XDG_RUNTIME_DIR absent : ouvre une session COSMIC"))?
        .join("nebula-paste");
    std::fs::create_dir_all(&dir)?;
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    Ok(dir)
}

pub struct Instance {
    pub socket: UnixDatagram,
    _lock: File,
    path: PathBuf,
}
impl Instance {
    pub fn acquire() -> io::Result<Self> {
        let dir = directory()?;
        let lock = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(dir.join("instance.lock"))?;
        lock.try_lock()
            .map_err(|_| io::Error::other("Nebula Paste fonctionne déjà ; utilise --toggle"))?;
        let path = dir.join("control.sock");
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        let socket = UnixDatagram::bind(&path)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            _lock: lock,
            path,
        })
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
pub fn toggle() -> io::Result<()> {
    let socket = UnixDatagram::unbound()?;
    socket.send_to(b"toggle", directory()?.join("control.sock"))?;
    Ok(())
}

/// Ask the applet to open its shared history window.
pub fn history() -> io::Result<()> {
    let socket = UnixDatagram::unbound()?;
    socket.send_to(b"history", directory()?.join("control.sock"))?;
    Ok(())
}
