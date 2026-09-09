use crate::model::{Clip, MAX_HISTORY_BYTES, MAX_ITEMS};
use rusqlite::{Connection, params};
use std::{
    fs::{self, OpenOptions},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};

pub struct Store {
    connection: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, String> {
        let parent = path.parent().ok_or("Chemin de données invalide")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| e.to_string())?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
        let connection = Connection::open(path).map_err(|e| e.to_string())?;
        Self::initialize(connection)
    }
    pub fn in_memory() -> Result<Self, String> {
        Self::initialize(Connection::open_in_memory().map_err(|e| e.to_string())?)
    }
    fn initialize(connection: Connection) -> Result<Self, String> {
        connection
            .execute_batch(
                "PRAGMA journal_mode=DELETE; PRAGMA secure_delete=ON;
            CREATE TABLE IF NOT EXISTS clips (
                id TEXT PRIMARY KEY, mime TEXT NOT NULL, bytes BLOB NOT NULL,
                timestamp INTEGER NOT NULL, pinned INTEGER NOT NULL DEFAULT 0,
                category TEXT NOT NULL DEFAULT ''
            ); CREATE INDEX IF NOT EXISTS by_time ON clips(timestamp DESC);",
            )
            .map_err(|e| e.to_string())?;
        Ok(Self { connection })
    }
    pub fn load(&self) -> Result<Vec<Clip>, String> {
        self.load_cached(&mut Vec::new())
    }
    /// Reuse already validated payloads, avoiding image decoding on each UI change.
    pub fn load_cached(&self, previous: &mut Vec<Clip>) -> Result<Vec<Clip>, String> {
        let mut stmt = self.connection.prepare("SELECT id,mime,timestamp,pinned,category FROM clips ORDER BY timestamp DESC,rowid DESC").map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, bool>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let metadata = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        // Validate new rows before consuming the cache, so errors preserve the UI.
        let mut added = std::collections::HashMap::new();
        for (id, mime, timestamp, _, _) in &metadata {
            if previous.iter().any(|c| &c.id == id) {
                continue;
            }
            let bytes = self
                .connection
                .query_row("SELECT bytes FROM clips WHERE id=?1", [id], |row| {
                    row.get::<_, Vec<u8>>(0)
                })
                .map_err(|e| e.to_string())?;
            let clip = Clip::new(mime.clone(), bytes, *timestamp)?;
            if &clip.id != id {
                return Err("L’intégrité d’une entrée de l’historique est invalide".into());
            }
            added.insert(id.clone(), clip);
        }
        let mut cache: std::collections::HashMap<_, _> =
            previous.drain(..).map(|c| (c.id.clone(), c)).collect();
        cache.extend(added);
        let mut result = Vec::new();
        for (id, _, timestamp, pinned, category) in metadata {
            if let Some(mut clip) = cache.remove(&id) {
                clip.timestamp = timestamp;
                clip.pinned = pinned;
                clip.category = category;
                result.push(clip);
            }
        }
        Ok(result)
    }
    pub fn insert(&mut self, clip: &Clip) -> Result<(), String> {
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO clips(id,mime,bytes,timestamp) VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET timestamp=excluded.timestamp", params![clip.id, clip.mime, clip.bytes, clip.timestamp]).map_err(|e| e.to_string())?;
        loop {
            let (count, size): (usize, usize) = tx
                .query_row(
                    "SELECT COUNT(*),COALESCE(SUM(length(bytes)),0) FROM clips",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|e| e.to_string())?;
            if count <= MAX_ITEMS && size <= MAX_HISTORY_BYTES {
                break;
            }
            let deleted = tx.execute("DELETE FROM clips WHERE id=(SELECT id FROM clips WHERE pinned=0 AND id<>?1 ORDER BY timestamp ASC,rowid ASC LIMIT 1)", [&clip.id]).map_err(|e| e.to_string())?;
            if deleted == 0 {
                return Err(
                    "Historique plein : retire des favoris pour libérer de la place".into(),
                );
            }
        }
        tx.commit().map_err(|e| e.to_string())
    }
    pub fn pin(&self, id: &str, pinned: bool) -> Result<(), String> {
        self.connection
            .execute(
                "UPDATE clips SET pinned=?1 WHERE id=?2",
                params![pinned, id],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    pub fn category(&self, id: &str, category: &str) -> Result<(), String> {
        self.connection
            .execute(
                "UPDATE clips SET category=?1 WHERE id=?2",
                params![category, id],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    pub fn delete(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM clips WHERE id=?1", [id])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    pub fn clear_unpinned(&self) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM clips WHERE pinned=0", [])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
