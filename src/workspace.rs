//! Transactional collection operations for clips and persistent notes.
use crate::{
    model::{Clip, MAX_HISTORY_BYTES, MAX_ITEMS},
    notes::{Archive, MAX_NOTES, Note},
    storage::Store,
    tr,
};
use rusqlite::{Connection, params};
use std::collections::HashSet;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Clip(String),
    Note(String),
}
impl Key {
    fn table(&self) -> &'static str {
        match self {
            Self::Clip(_) => "clips",
            Self::Note(_) => "notes",
        }
    }
    fn column(&self) -> &'static str {
        match self {
            Self::Clip(_) => "category",
            Self::Note(_) => "collection",
        }
    }
    fn id(&self) -> &str {
        match self {
            Self::Clip(s) | Self::Note(s) => s,
        }
    }
}
pub enum Undo {
    Moved {
        previous: Vec<(Key, String)>,
        destination: String,
    },
    Deleted {
        clips: Vec<Clip>,
        notes: Vec<Note>,
        ocr: Vec<(String, String, String)>,
    },
}
fn error() -> String {
    tr!(
        "Les éléments ont changé : actualise la sélection.",
        "Items changed: refresh the selection."
    )
    .into()
}
pub(crate) fn initialize(c: &Connection) -> Result<(), String> {
    c.execute_batch("CREATE TABLE IF NOT EXISTS collection_preferences(collection TEXT PRIMARY KEY, position INTEGER NOT NULL DEFAULT 2147483647, cards INTEGER NOT NULL DEFAULT 1);").map_err(|e|e.to_string())
}
impl Store {
    pub fn collection_cards(&self, name: &str) -> bool {
        self.connection
            .query_row(
                "SELECT cards FROM collection_preferences WHERE collection=?1",
                [name],
                |r| r.get(0),
            )
            .unwrap_or(true)
    }
    pub fn set_collection_cards(&self, name: &str, cards: bool) -> Result<(), String> {
        self.connection.execute("INSERT INTO collection_preferences(collection,cards) VALUES(?1,?2) ON CONFLICT(collection) DO UPDATE SET cards=excluded.cards",params![name,cards]).map(|_|()).map_err(|e|e.to_string())
    }
    pub fn move_collection(&self, name: &str, up: bool) -> Result<(), String> {
        let mut names = self.collections()?;
        let i = names.iter().position(|n| n == name).ok_or_else(error)?;
        let j = if up {
            i.saturating_sub(1)
        } else {
            (i + 1).min(names.len() - 1)
        };
        names.swap(i, j);
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        for (i, n) in names.iter().enumerate() {
            tx.execute("INSERT INTO collection_preferences(collection,position) VALUES(?1,?2) ON CONFLICT(collection) DO UPDATE SET position=excluded.position",params![n,i as i64]).map_err(|e|e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
    fn selection(&self, keys: &HashSet<Key>) -> Result<Vec<(Key, String)>, String> {
        if keys.is_empty() || keys.len() > MAX_ITEMS + MAX_NOTES {
            return Err(error());
        }
        keys.iter()
            .map(|k| {
                let value = self
                    .connection
                    .query_row(
                        &format!("SELECT {} FROM {} WHERE id=?1", k.column(), k.table()),
                        [k.id()],
                        |r| r.get::<_, String>(0),
                    )
                    .map_err(|_| error())?;
                Ok((k.clone(), value))
            })
            .collect()
    }
    pub fn move_items(&self, keys: &HashSet<Key>, destination: &str) -> Result<Undo, String> {
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        if !destination.is_empty() && !self.collections()?.iter().any(|n| n == destination) {
            return Err(error());
        }
        let previous = self.selection(keys)?;
        for (k, _) in &previous {
            tx.execute(
                &format!("UPDATE {} SET {}=?1 WHERE id=?2", k.table(), k.column()),
                params![destination, k.id()],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(Undo::Moved {
            previous,
            destination: destination.into(),
        })
    }
    pub fn delete_items(&self, keys: &HashSet<Key>) -> Result<Undo, String> {
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        self.selection(keys)?;
        let clips: Vec<_> = self
            .load()?
            .into_iter()
            .filter(|c| keys.contains(&Key::Clip(c.id.clone())))
            .collect();
        let notes: Vec<_> = self
            .notes()?
            .into_iter()
            .filter(|n| keys.contains(&Key::Note(n.id.clone())))
            .collect();
        let mut ocr = Vec::new();
        for c in &clips {
            let mut statement = tx
                .prepare("SELECT clip_id,language,text FROM image_text WHERE clip_id=?1")
                .map_err(|e| e.to_string())?;
            let rows = statement
                .query_map([&c.id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(|e| e.to_string())?;
            ocr.extend(
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?,
            );
        }
        for k in keys {
            tx.execute(&format!("DELETE FROM {} WHERE id=?1", k.table()), [k.id()])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(Undo::Deleted { clips, notes, ocr })
    }
    pub fn undo_items(&self, undo: &Undo) -> Result<(), String> {
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let collections = self.collections()?;
        let collection = |name: &str| {
            if collections.iter().any(|n| n == name) {
                name.to_owned()
            } else {
                String::new()
            }
        };
        match undo {
            Undo::Moved {
                previous,
                destination,
            } => {
                for (k, old) in previous {
                    let changed = tx
                        .execute(
                            &format!(
                                "UPDATE {} SET {}=?1 WHERE id=?2 AND {}=?3",
                                k.table(),
                                k.column(),
                                k.column()
                            ),
                            params![collection(old), k.id(), destination],
                        )
                        .map_err(|e| e.to_string())?;
                    if changed != 1 {
                        return Err(error());
                    }
                }
            }
            Undo::Deleted { clips, notes, ocr } => {
                let existing = self.load()?;
                let existing_notes = self.notes()?;
                if existing.len() + clips.len() > MAX_ITEMS
                    || existing
                        .iter()
                        .chain(clips.iter())
                        .map(|c| c.bytes.len())
                        .sum::<usize>()
                        > MAX_HISTORY_BYTES
                    || existing_notes.len() + notes.len() > MAX_NOTES
                {
                    return Err(tr!(
                        "Limite atteinte : annulation impossible sans supprimer d’autres données.",
                        "Limit reached: cannot undo without removing other data."
                    )
                    .into());
                }
                if clips.iter().any(|c| existing.iter().any(|e| e.id == c.id))
                    || notes
                        .iter()
                        .any(|n| existing_notes.iter().any(|e| e.id == n.id))
                {
                    return Err(error());
                }
                let mut all = existing_notes;
                all.extend(notes.iter().cloned());
                Archive::new(all).validate()?;
                for c in clips {
                    tx.execute("INSERT INTO clips(id,mime,bytes,timestamp,pinned,category) VALUES(?1,?2,?3,?4,?5,?6)",params![c.id,c.mime,c.bytes,c.timestamp,c.pinned,collection(&c.category)]).map_err(|e|e.to_string())?;
                }
                for n in notes {
                    tx.execute("INSERT INTO notes(id,title,body,collection,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6)",params![n.id,n.title,n.body,collection(&n.collection),n.created_at,n.updated_at]).map_err(|e|e.to_string())?;
                }
                for (id, lang, text) in ocr {
                    tx.execute(
                        "INSERT INTO image_text(clip_id,language,text) VALUES(?1,?2,?3)",
                        params![id, lang, text],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())
    }
}
