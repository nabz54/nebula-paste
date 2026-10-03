//! Persistent boards and local automation. All writes use bounded, validated data.
use crate::{
    model::{self, Kind},
    storage::Store,
};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
fn err(e: impl ToString) -> String {
    e.to_string()
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Column {
    pub id: i64,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rule {
    pub id: i64,
    pub kind: Option<Kind>,
    pub contains: String,
    pub destination: String,
    pub enabled: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    pub id: String,
    pub title: String,
    pub before: String,
    pub destinations: Vec<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Policy {
    pub days: u32,
    pub excluded: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct RuleUndo {
    changes: Vec<(String, String, String)>,
}
pub(crate) fn initialize(c: &Connection) -> Result<(), String> {
    c.execute_batch("CREATE TABLE IF NOT EXISTS board_columns(id INTEGER PRIMARY KEY, collection TEXT NOT NULL REFERENCES collections(name) ON UPDATE CASCADE ON DELETE CASCADE, name TEXT NOT NULL, position INTEGER NOT NULL, UNIQUE(collection,name));
    CREATE TABLE IF NOT EXISTS board_items(kind INTEGER NOT NULL, item TEXT NOT NULL, column_id INTEGER NOT NULL REFERENCES board_columns(id) ON DELETE CASCADE, PRIMARY KEY(kind,item));
    CREATE TABLE IF NOT EXISTS board_views(collection TEXT PRIMARY KEY REFERENCES collections(name) ON UPDATE CASCADE ON DELETE CASCADE);
    CREATE TABLE IF NOT EXISTS organize_rules(id INTEGER PRIMARY KEY, kind TEXT, contains TEXT NOT NULL, destination TEXT NOT NULL REFERENCES collections(name) ON UPDATE CASCADE ON DELETE CASCADE, enabled INTEGER NOT NULL DEFAULT 1);
    CREATE TABLE IF NOT EXISTS clip_usage(id TEXT PRIMARY KEY REFERENCES clips(id) ON DELETE CASCADE, used INTEGER NOT NULL);
    CREATE TABLE IF NOT EXISTS unused_policy(singleton INTEGER PRIMARY KEY CHECK(singleton=1), days INTEGER NOT NULL DEFAULT 0);
    CREATE TABLE IF NOT EXISTS unused_exclusions(collection TEXT PRIMARY KEY REFERENCES collections(name) ON UPDATE CASCADE ON DELETE CASCADE);
    CREATE TRIGGER IF NOT EXISTS board_clip_deleted AFTER DELETE ON clips BEGIN DELETE FROM board_items WHERE kind=0 AND item=OLD.id; END;
    CREATE TRIGGER IF NOT EXISTS board_note_deleted AFTER DELETE ON notes BEGIN DELETE FROM board_items WHERE kind=1 AND item=OLD.id; END;
    CREATE TRIGGER IF NOT EXISTS board_template_deleted AFTER DELETE ON templates BEGIN DELETE FROM board_items WHERE kind=2 AND item=OLD.id; END;").map_err(err)
}
impl Store {
    pub fn board_enabled(&self, collection: &str) -> bool {
        self.connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM board_views WHERE collection=?1)",
                [collection],
                |r| r.get(0),
            )
            .unwrap_or(false)
    }
    pub fn set_board_enabled(&self, collection: &str, enabled: bool) -> Result<(), String> {
        if enabled {
            self.connection
                .execute("INSERT OR IGNORE INTO board_views VALUES(?1)", [collection])
                .map_err(err)?;
        } else {
            self.connection
                .execute("DELETE FROM board_views WHERE collection=?1", [collection])
                .map_err(err)?;
        }
        Ok(())
    }
    pub fn board_columns(&self, collection: &str) -> Result<Vec<Column>, String> {
        self.connection
            .prepare("SELECT id,name FROM board_columns WHERE collection=?1 ORDER BY position,id")
            .map_err(err)?
            .query_map([collection], |r| {
                Ok(Column {
                    id: r.get(0)?,
                    name: r.get(1)?,
                })
            })
            .map_err(err)?
            .collect::<Result<_, _>>()
            .map_err(err)
    }
    pub fn save_column(&self, collection: &str, id: Option<i64>, name: &str) -> Result<(), String> {
        let name = Self::collection_name(name)?;
        if let Some(id) = id {
            if self
                .connection
                .execute(
                    "UPDATE board_columns SET name=?1 WHERE id=?2 AND collection=?3",
                    params![name, id, collection],
                )
                .map_err(err)?
                != 1
            {
                return Err(crate::tr!("La colonne a changé.", "Column changed.").into());
            }
        } else {
            if self.board_columns(collection)?.len() >= 12 {
                return Err(crate::tr!("12 colonnes maximum.", "12 columns maximum.").into());
            }
            self.connection.execute("INSERT INTO board_columns(collection,name,position) VALUES(?1,?2,COALESCE((SELECT MAX(position)+1 FROM board_columns WHERE collection=?1),0))",params![collection,name]).map_err(err)?;
        }
        Ok(())
    }
    pub fn delete_column(&self, id: i64) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM board_columns WHERE id=?1", [id])
            .map(|_| ())
            .map_err(err)
    }
    pub fn order_column(&self, collection: &str, id: i64, up: bool) -> Result<(), String> {
        let mut cols = self.board_columns(collection)?;
        let i = cols
            .iter()
            .position(|c| c.id == id)
            .ok_or(crate::tr!("Colonne introuvable.", "Column missing."))?;
        let j = if up {
            i.saturating_sub(1)
        } else {
            (i + 1).min(cols.len() - 1)
        };
        cols.swap(i, j);
        let tx = self.connection.unchecked_transaction().map_err(err)?;
        for (i, c) in cols.iter().enumerate() {
            tx.execute(
                "UPDATE board_columns SET position=?1 WHERE id=?2",
                params![i as i64, c.id],
            )
            .map_err(err)?;
        }
        tx.commit().map_err(err)
    }
    pub fn board_assignments(
        &self,
        collection: &str,
    ) -> Result<HashMap<(u8, String), i64>, String> {
        self.connection.prepare("SELECT b.kind,b.item,b.column_id FROM board_items b JOIN board_columns c ON c.id=b.column_id WHERE c.collection=?1").map_err(err)?.query_map([collection],|r|Ok(((r.get(0)?,r.get(1)?),r.get(2)?))).map_err(err)?.collect::<Result<_,_>>().map_err(err)
    }
    pub fn board_move(
        &self,
        collection: &str,
        kind: u8,
        item: &str,
        column: Option<i64>,
    ) -> Result<(), String> {
        let (table, col) = match kind {
            0 => ("clips", "category"),
            1 => ("notes", "collection"),
            2 => ("templates", "collection"),
            _ => return Err(crate::tr!("Type d’élément invalide.", "Invalid item type.").into()),
        };
        let tx = self.connection.unchecked_transaction().map_err(err)?;
        let current: String = tx
            .query_row(
                &format!("SELECT {col} FROM {table} WHERE id=?1"),
                [item],
                |r| r.get(0),
            )
            .map_err(err)?;
        if current != collection || collection.is_empty() {
            return Err(crate::tr!(
                "Élément déplacé : actualise le tableau.",
                "Item moved: refresh board."
            )
            .into());
        }
        if let Some(column) = column {
            let owner: String = tx
                .query_row(
                    "SELECT collection FROM board_columns WHERE id=?1",
                    [column],
                    |r| r.get(0),
                )
                .map_err(err)?;
            if owner != collection {
                return Err(crate::tr!(
                    "La colonne appartient à une autre collection.",
                    "Column belongs to another collection."
                )
                .into());
            }
            tx.execute("INSERT INTO board_items VALUES(?1,?2,?3) ON CONFLICT(kind,item) DO UPDATE SET column_id=excluded.column_id",params![kind,item,column]).map_err(err)?;
        } else {
            tx.execute(
                "DELETE FROM board_items WHERE kind=?1 AND item=?2",
                params![kind, item],
            )
            .map_err(err)?;
        }
        tx.commit().map_err(err)
    }
    pub fn rules(&self) -> Result<Vec<Rule>, String> {
        let rows: Vec<(i64, Option<String>, String, String, bool)> = self
            .connection
            .prepare("SELECT id,kind,contains,destination,enabled FROM organize_rules ORDER BY id")
            .map_err(err)?
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .map_err(err)?
            .collect::<Result<_, _>>()
            .map_err(err)?;
        rows.into_iter()
            .map(|(id, kind, contains, destination, enabled)| {
                Ok(Rule {
                    id,
                    kind: kind
                        .map(|s| serde_json::from_str(&s).map_err(err))
                        .transpose()?,
                    contains,
                    destination,
                    enabled,
                })
            })
            .collect()
    }
    pub fn save_rule(&self, rule: &Rule) -> Result<(), String> {
        if (rule.contains.trim().is_empty() && rule.kind.is_none())
            || rule.contains.len() > 256
            || rule.contains.contains('\0')
        {
            return Err(crate::tr!(
                "Choisis un type ou un texte recherché (256 octets maximum).",
                "Choose a type or matching text (maximum 256 bytes)."
            )
            .into());
        }
        if rule.id == 0 && self.rules()?.len() >= 64 {
            return Err(crate::tr!("64 règles maximum.", "64 rules maximum.").into());
        }
        let kind = rule
            .kind
            .map(|k| serde_json::to_string(&k))
            .transpose()
            .map_err(err)?;
        if rule.id==0 {self.connection.execute("INSERT INTO organize_rules(kind,contains,destination,enabled) VALUES(?1,?2,?3,?4)",params![kind,rule.contains.trim(),rule.destination,rule.enabled]).map_err(err)?;}
        else if self.connection.execute("UPDATE organize_rules SET kind=?1,contains=?2,destination=?3,enabled=?4 WHERE id=?5",params![kind,rule.contains.trim(),rule.destination,rule.enabled,rule.id]).map_err(err)?!=1 {return Err(crate::tr!("Règle introuvable.", "Rule missing.").into());}
        Ok(())
    }
    pub fn delete_rule(&self, id: i64) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM organize_rules WHERE id=?1", [id])
            .map(|_| ())
            .map_err(err)
    }
    pub fn rule_preview(&self) -> Result<Vec<Match>, String> {
        let rules = self.rules()?;
        let mut matches = Vec::new();
        for c in self.load()? {
            if c.pinned {
                continue;
            }
            let hay = model::search_fold(&c.text);
            let mut destinations: Vec<String> = rules
                .iter()
                .filter(|r| {
                    r.enabled
                        && r.kind.is_none_or(|k| k == c.kind)
                        && hay.contains(&model::search_fold(&r.contains))
                })
                .map(|r| r.destination.clone())
                .collect();
            destinations.sort();
            destinations.dedup();
            if !destinations.is_empty()
                && !(destinations.len() == 1 && destinations[0] == c.category)
            {
                matches.push(Match {
                    id: c.id,
                    title: c.title,
                    before: c.category,
                    destinations,
                });
            }
        }
        Ok(matches)
    }
    pub fn apply_rules(&self, preview: &[Match]) -> Result<RuleUndo, String> {
        let tx = self.connection.unchecked_transaction().map_err(err)?;
        if self.rule_preview()? != preview {
            return Err(crate::tr!(
                "L’aperçu a changé : actualise avant application.",
                "Preview changed: refresh before applying."
            )
            .into());
        }
        let mut changes = Vec::new();
        for m in preview {
            if m.destinations.len() != 1 {
                continue;
            }
            let dest = &m.destinations[0];
            tx.execute(
                "UPDATE clips SET category=?1 WHERE id=?2",
                params![dest, m.id],
            )
            .map_err(err)?;
            changes.push((m.id.clone(), m.before.clone(), dest.clone()));
        }
        tx.commit().map_err(err)?;
        Ok(RuleUndo { changes })
    }
    pub fn undo_rules(&self, undo: &RuleUndo) -> Result<(), String> {
        let tx = self.connection.unchecked_transaction().map_err(err)?;
        for (id, before, after) in &undo.changes {
            if !before.is_empty() && !self.collections()?.contains(before) {
                return Err(crate::tr!(
                    "Collection d’origine introuvable.",
                    "Original collection missing."
                )
                .into());
            }
            if tx
                .execute(
                    "UPDATE clips SET category=?1 WHERE id=?2 AND category=?3",
                    params![before, id, after],
                )
                .map_err(err)?
                != 1
            {
                return Err(crate::tr!(
                    "Élément modifié : annulation refusée.",
                    "Item changed: undo refused."
                )
                .into());
            }
        }
        tx.commit().map_err(err)
    }
    /// Apply only unambiguous rules to newly captured, unfiled, unpinned clips.
    pub fn classify_new(&self, id: &str) -> Result<(), String> {
        let p = self.rule_preview()?;
        if let Some(m) = p
            .iter()
            .find(|m| m.id == id && m.before.is_empty() && m.destinations.len() == 1)
        {
            self.category(id, &m.destinations[0])?;
        }
        Ok(())
    }
    pub fn mark_used(&self, id: &str, now: i64) -> Result<(), String> {
        self.connection.execute("INSERT INTO clip_usage(id,used) SELECT id,?2 FROM clips WHERE id=?1 ON CONFLICT(id) DO UPDATE SET used=MAX(used,excluded.used)",params![id,now]).map(|_|()).map_err(err)
    }
    pub fn unused_policy(&self) -> Result<Policy, String> {
        let days = self
            .connection
            .query_row(
                "SELECT COALESCE((SELECT days FROM unused_policy WHERE singleton=1),0)",
                [],
                |r| r.get(0),
            )
            .map_err(err)?;
        let excluded = self
            .connection
            .prepare("SELECT collection FROM unused_exclusions ORDER BY collection")
            .map_err(err)?
            .query_map([], |r| r.get(0))
            .map_err(err)?
            .collect::<Result<_, _>>()
            .map_err(err)?;
        Ok(Policy { days, excluded })
    }
    pub fn save_unused_policy(&self, p: &Policy) -> Result<(), String> {
        if p.days > 3650 || p.excluded.len() > 128 {
            return Err(crate::tr!(
                "Paramètres d’expiration invalides.",
                "Invalid expiration policy."
            )
            .into());
        }
        let tx = self.connection.unchecked_transaction().map_err(err)?;
        tx.execute("INSERT INTO unused_policy VALUES(1,?1) ON CONFLICT(singleton) DO UPDATE SET days=excluded.days",[p.days]).map_err(err)?;
        tx.execute("DELETE FROM unused_exclusions", [])
            .map_err(err)?;
        for c in &p.excluded {
            tx.execute("INSERT OR IGNORE INTO unused_exclusions VALUES(?1)", [c])
                .map_err(err)?;
        }
        tx.commit().map_err(err)
    }
    pub fn unused_preview(&self, p: &Policy, now: i64) -> Result<Vec<(String, String)>, String> {
        if p.days == 0 {
            return Ok(vec![]);
        }
        if p.days > 3650 {
            return Err(
                crate::tr!("Délai d’expiration invalide.", "Invalid expiration days.").into(),
            );
        }
        let cutoff = now.saturating_sub(i64::from(p.days) * 86400);
        let excluded: HashSet<_> = p.excluded.iter().collect();
        let rows:Vec<(String,String)>=self.connection.prepare("SELECT id,category FROM clips WHERE pinned=0 AND MAX(timestamp,COALESCE((SELECT used FROM clip_usage WHERE clip_usage.id=clips.id),timestamp))<?1 ORDER BY id").map_err(err)?.query_map([cutoff],|r|Ok((r.get(0)?,r.get(1)?))).map_err(err)?.collect::<Result<_,_>>().map_err(err)?;
        Ok(rows
            .into_iter()
            .filter(|(_, c)| !excluded.contains(c))
            .collect())
    }
    pub fn expire_unused(&self, now: i64) -> Result<usize, String> {
        let tx = self.connection.unchecked_transaction().map_err(err)?;
        let p = self.unused_policy()?;
        let rows = self.unused_preview(&p, now)?;
        for (id, _) in &rows {
            tx.execute("DELETE FROM clips WHERE id=?1", [id])
                .map_err(err)?;
        }
        tx.commit().map_err(err)?;
        Ok(rows.len())
    }
}
