use crate::model::{Clip, MAX_HISTORY_BYTES, MAX_ITEMS};
use rusqlite::{Connection, params};
use std::collections::HashMap;

pub const MAX_INDEX_CHARS: usize = 16_384;
pub const MAX_COLLECTIONS: usize = 128;
use std::{
    fs::{self, OpenOptions},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};

pub struct Store {
    pub(crate) connection: Connection,
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
                "PRAGMA journal_mode=DELETE; PRAGMA secure_delete=ON; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS clips (
                id TEXT PRIMARY KEY, mime TEXT NOT NULL, bytes BLOB NOT NULL,
                timestamp INTEGER NOT NULL, pinned INTEGER NOT NULL DEFAULT 0,
                category TEXT NOT NULL DEFAULT ''
            ); CREATE INDEX IF NOT EXISTS by_time ON clips(timestamp DESC);
            CREATE TABLE IF NOT EXISTS collections (name TEXT PRIMARY KEY NOT NULL);
            INSERT OR IGNORE INTO collections(name) SELECT DISTINCT category FROM clips WHERE category<>'';
            CREATE TABLE IF NOT EXISTS image_text (
                clip_id TEXT PRIMARY KEY REFERENCES clips(id) ON DELETE CASCADE,
                language TEXT NOT NULL, text TEXT NOT NULL
            );",
            )
            .map_err(|e| e.to_string())?;
        crate::templates::initialize(&connection)?;
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
        let previous_ids: std::collections::HashSet<_> =
            previous.iter().map(|c| c.id.as_str()).collect();
        let mut added = std::collections::HashMap::new();
        for (id, mime, timestamp, _, _) in &metadata {
            if previous_ids.contains(id.as_str()) {
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
    pub fn collection_name(name: &str) -> Result<String, String> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 40 || name.chars().any(char::is_control) {
            return Err(crate::tr!(
                "Nom requis : 1 à 40 caractères, sans contrôle.",
                "Name required: 1–40 characters, no control characters."
            )
            .into());
        }
        Ok(name.to_owned())
    }
    pub fn collections(&self) -> Result<Vec<String>, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT name FROM collections ORDER BY name COLLATE NOCASE")
            .map_err(|e| e.to_string())?;
        stmt.query_map([], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
    pub fn create_collection(&self, name: &str) -> Result<(), String> {
        let name = Self::collection_name(name)?;
        if self.collections()?.len() >= MAX_COLLECTIONS {
            return Err(
                crate::tr!("Limite de collections atteinte", "Collection limit reached").into(),
            );
        }
        self.connection
            .execute("INSERT INTO collections(name) VALUES(?1)", [name])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    pub fn rename_collection(&self, old: &str, name: &str) -> Result<(), String> {
        let name = Self::collection_name(name)?;
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let changed = tx
            .execute(
                "UPDATE collections SET name=?1 WHERE name=?2",
                params![name, old],
            )
            .map_err(|e| e.to_string())?;
        if changed != 1 {
            return Err(crate::tr!("Collection introuvable", "Collection not found").into());
        }
        tx.execute(
            "UPDATE clips SET category=?1 WHERE category=?2",
            params![name, old],
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE templates SET collection=?1 WHERE collection=?2",
            params![name, old],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }
    /// Deleting a collection only unfiles its clips, including favorites.
    pub fn delete_collection(&self, name: &str) -> Result<(), String> {
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        tx.execute("UPDATE clips SET category='' WHERE category=?1", [name])
            .map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE templates SET collection='' WHERE collection=?1",
            [name],
        )
        .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM collections WHERE name=?1", [name])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }
    pub fn category(&self, id: &str, category: &str) -> Result<(), String> {
        let category = if category.trim().is_empty() {
            String::new()
        } else {
            Self::collection_name(category)?
        };
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM clips WHERE id=?1)",
                [id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !exists {
            return Err(crate::tr!("Copie introuvable", "Clip not found").into());
        }
        if !category.is_empty() {
            let known: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM collections WHERE name=?1)",
                    [&category],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            if !known {
                let count: usize = tx
                    .query_row("SELECT COUNT(*) FROM collections", [], |r| r.get(0))
                    .map_err(|e| e.to_string())?;
                if count >= MAX_COLLECTIONS {
                    return Err(crate::tr!(
                        "Limite de collections atteinte",
                        "Collection limit reached"
                    )
                    .into());
                }
                tx.execute("INSERT INTO collections(name) VALUES(?1)", [&category])
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.execute(
            "UPDATE clips SET category=?1 WHERE id=?2",
            params![category, id],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }
    /// The derived text never changes the source bytes or creates a clipboard entry.
    pub fn index_image(&self, id: &str, language: &str, text: &str) -> Result<bool, String> {
        if !crate::settings::OCR_LANGUAGES.contains(&language) {
            return Err("Invalid OCR language".into());
        }
        let bounded: String = text.chars().take(MAX_INDEX_CHARS).collect();
        self.connection
            .execute(
                "INSERT INTO image_text(clip_id,language,text)
            SELECT id,?2,?3 FROM clips WHERE id=?1 AND mime LIKE 'image/%'
            ON CONFLICT(clip_id) DO UPDATE SET language=excluded.language,text=excluded.text",
                params![id, language, bounded],
            )
            .map(|n| n > 0)
            .map_err(|e| e.to_string())
    }
    pub fn image_index(&self, language: &str) -> Result<HashMap<String, String>, String> {
        let mut stmt = self
            .connection
            .prepare("SELECT clip_id,text FROM image_text WHERE language=?1")
            .map_err(|e| e.to_string())?;
        stmt.query_map([language], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<HashMap<_, _>, _>>()
            .map_err(|e| e.to_string())
    }
    pub fn clear_image_index(&self) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM image_text", [])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    pub fn delete(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM clips WHERE id=?1", [id])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    /// Supprime les entrées non favorites antérieures à `cutoff`. Les favoris ne
    /// sont jamais concernés par la durée de rétention.
    pub fn purge_older_than(&self, cutoff: i64) -> Result<usize, String> {
        self.connection
            .execute(
                "DELETE FROM clips WHERE pinned=0 AND timestamp<?1",
                [cutoff],
            )
            .map_err(|e| e.to_string())
    }
    /// Undo preserves metadata and never evicts other clips or overwrites a recaptured clip.
    pub fn restore(&mut self, clip: &Clip) -> Result<bool, String> {
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        let inserted = tx.execute("INSERT INTO clips(id,mime,bytes,timestamp,pinned,category) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO NOTHING", params![clip.id, clip.mime, clip.bytes, clip.timestamp, clip.pinned, clip.category]).map_err(|e| e.to_string())?;
        if inserted != 0 && !clip.category.is_empty() {
            let exists: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM collections WHERE name=?1)",
                    [&clip.category],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            if !exists {
                Self::collection_name(&clip.category)?;
                let count: usize = tx
                    .query_row("SELECT COUNT(*) FROM collections", [], |row| row.get(0))
                    .map_err(|e| e.to_string())?;
                if count >= MAX_COLLECTIONS {
                    return Err(crate::tr!("Trop de collections", "Too many collections").into());
                }
            }
            tx.execute(
                "INSERT OR IGNORE INTO collections(name) VALUES(?1)",
                [&clip.category],
            )
            .map_err(|e| e.to_string())?;
        }
        let (count, size): (usize, usize) = tx
            .query_row(
                "SELECT COUNT(*),COALESCE(SUM(length(bytes)),0) FROM clips",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        if count > MAX_ITEMS || size > MAX_HISTORY_BYTES {
            return Err("Impossible de restaurer : historique plein".into());
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(inserted != 0)
    }
    pub fn clear_unpinned(&self) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM clips WHERE pinned=0", [])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
