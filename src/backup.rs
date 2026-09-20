//! Bounded portable history backups, atomic restoration, content-free diagnostics.
use crate::{
    model::{Clip, MAX_CLIP_BYTES, MAX_HISTORY_BYTES, MAX_ITEMS},
    settings::{OCR_LANGUAGES, Settings},
    storage::{MAX_COLLECTIONS, MAX_INDEX_CHARS, Store},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub const MAX_ARCHIVE_BYTES: usize = 256 * 1024 * 1024;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub mime: String,
    pub data: String,
    pub timestamp: i64,
    pub pinned: bool,
    pub collection: String,
    pub ocr: Option<Ocr>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ocr {
    pub language: String,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    pub format: String,
    pub version: u32,
    pub collections: Vec<String>,
    pub clips: Vec<Entry>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conflict {
    KeepLocal,
    UseBackup,
}
#[derive(Debug, Default, Clone, Copy)]
pub struct Summary {
    pub added: usize,
    pub conflicts: usize,
    pub pinned: usize,
    pub bytes: usize,
}
#[derive(Debug)]
pub struct Validated {
    archive: Archive,
    decoded: Vec<Vec<u8>>,
}
impl Validated {
    pub fn summary(&self, ids: &HashSet<String>) -> Summary {
        let conflicts = self
            .archive
            .clips
            .iter()
            .filter(|c| ids.contains(&c.id))
            .count();
        Summary {
            added: self.archive.clips.len() - conflicts,
            conflicts,
            pinned: self.archive.clips.iter().filter(|c| c.pinned).count(),
            bytes: self.decoded.iter().map(Vec::len).sum(),
        }
    }
    pub fn collections(&self) -> usize {
        self.archive.collections.len()
    }
}
impl Archive {
    pub fn validate(self) -> Result<Validated, String> {
        if self.format != "nebula-paste-history" || self.version != 1 {
            return Err("Unsupported history archive format/version".into());
        }
        if self.clips.len() > MAX_ITEMS || self.collections.len() > MAX_COLLECTIONS {
            return Err("Archive item limit exceeded".into());
        }
        let mut names = HashSet::new();
        for n in &self.collections {
            if Store::collection_name(n)? != *n || !names.insert(n.as_str()) {
                return Err("Invalid or duplicate collection".into());
            }
        }
        let mut ids = HashSet::new();
        let mut decoded = Vec::new();
        let mut size = 0usize;
        for e in &self.clips {
            if !ids.insert(e.id.as_str())
                || e.timestamp < 0
                || e.mime.len() > 256
                || e.data.len() > MAX_CLIP_BYTES.div_ceil(3) * 4
            {
                return Err("Invalid entry".into());
            }
            if !e.collection.is_empty() && !names.contains(e.collection.as_str()) {
                return Err("Missing collection".into());
            }
            let bytes = STANDARD
                .decode(&e.data)
                .map_err(|_| "Invalid base64 data")?;
            size = size.checked_add(bytes.len()).ok_or("Archive too large")?;
            if size > MAX_HISTORY_BYTES {
                return Err("History byte limit exceeded".into());
            }
            let c = Clip::new(e.mime.clone(), bytes.clone(), e.timestamp)?;
            if c.id != e.id {
                return Err("Content hash mismatch".into());
            }
            if let Some(o) = &e.ocr {
                if c.kind != crate::model::Kind::Image
                    || !OCR_LANGUAGES.contains(&o.language.as_str())
                    || o.text.chars().count() > MAX_INDEX_CHARS
                {
                    return Err("Invalid OCR index".into());
                }
            }
            decoded.push(bytes);
        }
        Ok(Validated {
            archive: self,
            decoded,
        })
    }
    pub fn read(path: &Path) -> Result<Validated, String> {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        if !file.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("Regular file required".into());
        }
        let mut bytes = Vec::new();
        file.take((MAX_ARCHIVE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > MAX_ARCHIVE_BYTES {
            return Err("Archive exceeds 256 MiB".into());
        }
        serde_json::from_slice::<Self>(&bytes)
            .map_err(|e| e.to_string())?
            .validate()
    }
    pub fn export_new(&self, path: &Path) -> Result<(), String> {
        let bytes = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_ARCHIVE_BYTES {
            return Err("Archive too large".into());
        }
        write_new(path, &bytes)
    }
}
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut f = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    f.write_all(bytes).map_err(|e| e.to_string())?;
    f.as_file().sync_all().map_err(|e| e.to_string())?;
    f.persist_noclobber(path).map_err(|e| e.to_string())?;
    std::fs::File::open(parent)
        .and_then(|d| d.sync_all())
        .map_err(|e| e.to_string())
}
impl Store {
    pub fn data_path(&self) -> Option<PathBuf> {
        self.connection
            .path()
            .filter(|p| !p.is_empty())
            .map(PathBuf::from)
    }
    pub fn export_snapshot(path: &Path, diagnostic: Option<&Settings>) -> Result<Vec<u8>, String> {
        let connection =
            rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(|e| e.to_string())?;
        connection
            .busy_timeout(std::time::Duration::from_secs(2))
            .map_err(|e| e.to_string())?;
        let s = Self { connection };
        let tx = s
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let result = if let Some(settings) = diagnostic {
            s.diagnostic(settings).map(String::into_bytes)
        } else {
            s.history_archive()
                .and_then(|a| serde_json::to_vec(&a).map_err(|e| e.to_string()))
        };
        drop(tx);
        let bytes = result?;
        if bytes.len() > MAX_ARCHIVE_BYTES {
            return Err("Archive exceeds 256 MiB".into());
        }
        Ok(bytes)
    }
    pub fn history_archive(&self) -> Result<Archive, String> {
        let mut stmt=self.connection.prepare("SELECT c.id,c.mime,c.bytes,c.timestamp,c.pinned,c.category,i.language,i.text FROM clips c LEFT JOIN image_text i ON c.id=i.clip_id ORDER BY c.timestamp DESC,c.id").map_err(|e|e.to_string())?;
        let clips = stmt
            .query_map([], |r| {
                let language: Option<String> = r.get(6)?;
                Ok(Entry {
                    id: r.get(0)?,
                    mime: r.get(1)?,
                    data: STANDARD.encode(r.get::<_, Vec<u8>>(2)?),
                    timestamp: r.get(3)?,
                    pinned: r.get(4)?,
                    collection: r.get(5)?,
                    ocr: match language {
                        Some(language) => Some(Ocr {
                            language,
                            text: r.get(7)?,
                        }),
                        None => None,
                    },
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(Archive {
            format: "nebula-paste-history".into(),
            version: 1,
            collections: self.collections()?,
            clips,
        })
    }
    pub fn restore_history(
        &mut self,
        data: &Validated,
        policy: Conflict,
        include_ocr: bool,
    ) -> Result<Summary, String> {
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        let mut result = Summary::default();
        for n in &data.archive.collections {
            tx.execute("INSERT OR IGNORE INTO collections(name) VALUES(?1)", [n])
                .map_err(|e| e.to_string())?;
        }
        for (e, bytes) in data.archive.clips.iter().zip(&data.decoded) {
            let exists: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM clips WHERE id=?1)",
                    [&e.id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            if exists {
                result.conflicts += 1;
                if policy == Conflict::KeepLocal {
                    continue;
                }
            } else {
                result.added += 1;
            }
            tx.execute("INSERT INTO clips(id,mime,bytes,timestamp,pinned,category) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET timestamp=excluded.timestamp,pinned=excluded.pinned,category=excluded.category",params![e.id,e.mime,bytes,e.timestamp,e.pinned,e.collection]).map_err(|e|e.to_string())?;
            tx.execute("DELETE FROM image_text WHERE clip_id=?1", [&e.id])
                .map_err(|e| e.to_string())?;
            if include_ocr && let Some(o) = &e.ocr {
                tx.execute(
                    "INSERT INTO image_text(clip_id,language,text) VALUES(?1,?2,?3)",
                    params![e.id, o.language, o.text],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        let (count, size): (usize, usize) = tx
            .query_row(
                "SELECT COUNT(*),COALESCE(SUM(length(bytes)),0) FROM clips",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        let collections: usize = tx
            .query_row("SELECT COUNT(*) FROM collections", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if count > MAX_ITEMS || size > MAX_HISTORY_BYTES || collections > MAX_COLLECTIONS {
            return Err("Restore exceeds capacity; no data changed".into());
        }
        result.bytes = size;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(result)
    }
    pub fn diagnostic(&self, settings: &Settings) -> Result<String, String> {
        let (clips,bytes,favorites):(u64,u64,u64)=self.connection.query_row("SELECT COUNT(*),COALESCE(SUM(length(bytes)),0),COALESCE(SUM(pinned<>0),0) FROM clips",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|e.to_string())?;
        let templates: u64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM templates", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let collections: u64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM collections", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let indexed: u64 = self
            .connection
            .query_row("SELECT COUNT(*) FROM image_text", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&serde_json::json!({"format":"nebula-paste-diagnostic","version":1,"app_version":env!("CARGO_PKG_VERSION"),"os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"clips":clips,"payload_bytes":bytes,"favorites":favorites,"templates":templates,"collections":collections,"indexed_images":indexed,"retention_days":settings.retention_days,"ocr_indexing":settings.ocr_indexing,"ocr_language":settings.ocr_language})).map_err(|e|e.to_string())
    }
}
