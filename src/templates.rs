//! Reusable text templates. Values are substituted literally, never evaluated.
use crate::{model, storage::Store, tr};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read, Write},
    path::Path,
};

pub const MAX_TEMPLATES: usize = 500;
pub const MAX_BODY: usize = 65_536;
pub const MAX_OUTPUT: usize = 262_144;
pub const MAX_ARCHIVE: usize = 16 * 1024 * 1024;
pub const MAX_FIELDS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub id: String,
    pub title: String,
    pub body: String,
    pub collection: String,
    pub created_at: i64,
    pub updated_at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    pub format: String,
    pub version: u32,
    pub templates: Vec<Template>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conflict {
    Skip,
    KeepBoth,
    Replace,
}
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportResult {
    pub added: usize,
    pub replaced: usize,
    pub skipped: usize,
}

fn invalid() -> String {
    tr!(
        "Modèle invalide : vérifie le titre, le contenu et les champs.",
        "Invalid template: check title, body and fields."
    )
    .into()
}

/// Ordered unique fields. Doubled braces are reserved; single braces are literal.
pub fn fields(body: &str) -> Result<Vec<String>, String> {
    if body.len() > MAX_BODY || body.contains('\0') {
        return Err(invalid());
    }
    let mut names = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("{{") {
        if rest[..start].contains("}}") {
            return Err(invalid());
        }
        rest = &rest[start + 2..];
        let end = rest.find("}}").ok_or_else(invalid)?;
        let name = &rest[..end];
        if name.is_empty()
            || name.chars().count() > 40
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(invalid());
        }
        if !names.iter().any(|n| n == name) {
            names.push(name.to_owned());
        }
        if names.len() > MAX_FIELDS {
            return Err(invalid());
        }
        rest = &rest[end + 2..];
    }
    if rest.contains("}}") {
        return Err(invalid());
    }
    Ok(names)
}

pub fn expand(body: &str, values: &HashMap<String, String>) -> Result<String, String> {
    for name in fields(body)? {
        if !values
            .get(&name)
            .is_some_and(|v| !v.trim().is_empty() && !v.contains('\0'))
        {
            return Err(tr!(
                "Remplis tous les champs avant de copier.",
                "Fill in every field before copying."
            )
            .into());
        }
    }
    let mut output = String::new();
    let mut rest = body;
    while let Some(start) = rest.find("{{") {
        let tail = &rest[start + 2..];
        let end = tail.find("}}").ok_or_else(invalid)?;
        let value = &values[&tail[..end]];
        if output.len() + start + value.len() > MAX_OUTPUT {
            return Err(invalid());
        }
        output.push_str(&rest[..start]);
        output.push_str(value);
        rest = &tail[end + 2..];
    }
    if output.len() + rest.len() > MAX_OUTPUT {
        return Err(invalid());
    }
    output.push_str(rest);
    Ok(output)
}
impl Template {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.len() != 32
            || !self.id.bytes().all(|b| b.is_ascii_hexdigit())
            || self.title.trim().is_empty()
            || self.title.chars().count() > 100
            || self.title.chars().any(char::is_control)
            || self.body.trim().is_empty()
            || self.created_at < 0
            || self.updated_at < self.created_at
        {
            return Err(invalid());
        }
        if !self.collection.is_empty()
            && Store::collection_name(&self.collection)? != self.collection
        {
            return Err(invalid());
        }
        fields(&self.body)?;
        Ok(())
    }
    pub fn matches(&self, query: &str, collection: &str) -> bool {
        if !collection.is_empty() && self.collection != collection {
            return false;
        }
        let haystack = model::search_fold(&format!(
            "{}\n{}\n{}",
            self.title, self.body, self.collection
        ));
        model::search_fold(query)
            .split_whitespace()
            .all(|word| haystack.contains(word))
    }
}
impl Archive {
    pub fn new(templates: Vec<Template>) -> Self {
        Self {
            format: "nebula-paste-templates".into(),
            version: 1,
            templates,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "nebula-paste-templates"
            || self.version != 1
            || self.templates.len() > MAX_TEMPLATES
        {
            return Err(invalid());
        }
        let mut ids = HashSet::new();
        for t in &self.templates {
            t.validate()?;
            if !ids.insert(&t.id) {
                return Err(invalid());
            }
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_ARCHIVE {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self, String> {
        if !fs::metadata(path).map_err(|e| e.to_string())?.is_file() {
            return Err(invalid());
        }
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take((MAX_ARCHIVE + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > MAX_ARCHIVE {
            return Err(invalid());
        }
        let result: Self = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        result.validate()?;
        Ok(result)
    }
    /// A new file only: even if a target appears after the chooser, never overwrite it.
    pub fn export_new(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|e| e.to_string())?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        file.persist_noclobber(path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub(crate) fn initialize(c: &Connection) -> Result<(), String> {
    c.execute_batch(
        "CREATE TABLE IF NOT EXISTS templates (
      id TEXT PRIMARY KEY, title TEXT NOT NULL, body TEXT NOT NULL,
      collection TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
    );",
    )
    .map_err(|e| e.to_string())
}
fn new_id(c: &Connection) -> Result<String, String> {
    c.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))
        .map_err(|e| e.to_string())
}
fn ensure_collection(c: &Connection, name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Ok(());
    }
    Store::collection_name(name)?;
    let exists: bool = c
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM collections WHERE name=?1)",
            [name],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if !exists {
        let count: usize = c
            .query_row("SELECT count(*) FROM collections", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if count >= crate::storage::MAX_COLLECTIONS {
            return Err(tr!("Trop de collections", "Too many collections").into());
        }
        c.execute("INSERT INTO collections(name) VALUES(?1)", [name])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn write(c: &Connection, t: &Template) -> Result<(), String> {
    ensure_collection(c, &t.collection)?;
    c.execute("INSERT INTO templates(id,title,body,collection,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6)
      ON CONFLICT(id) DO UPDATE SET title=excluded.title,body=excluded.body,collection=excluded.collection,updated_at=excluded.updated_at",
      params![t.id,t.title,t.body,t.collection,t.created_at,t.updated_at]).map(|_| ()).map_err(|e| e.to_string())
}
impl Store {
    pub fn templates(&self) -> Result<Vec<Template>, String> {
        let mut stmt = self.connection.prepare("SELECT id,title,body,collection,created_at,updated_at FROM templates ORDER BY title COLLATE NOCASE,id").map_err(|e| e.to_string())?;
        stmt.query_map([], |r| {
            Ok(Template {
                id: r.get(0)?,
                title: r.get(1)?,
                body: r.get(2)?,
                collection: r.get(3)?,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
    }
    pub fn save_template(
        &self,
        id: Option<&str>,
        title: &str,
        body: &str,
        collection: &str,
    ) -> Result<String, String> {
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let existing = self.templates()?;
        let old = id.and_then(|id| existing.iter().find(|t| t.id == id));
        if id.is_some() && old.is_none() {
            return Err(tr!("Modèle introuvable", "Template not found").into());
        }
        if id.is_none() && existing.len() >= MAX_TEMPLATES {
            return Err(tr!("Limite de modèles atteinte", "Template limit reached").into());
        }
        let t = Template {
            id: old.map(|t| t.id.clone()).unwrap_or(new_id(&tx)?),
            title: title.trim().into(),
            body: body.into(),
            collection: collection.trim().into(),
            created_at: old.map_or(model::now(), |t| t.created_at),
            updated_at: model::now().max(old.map_or(0, |t| t.created_at)),
        };
        t.validate()?;
        write(&tx, &t)?;
        // Keep the complete library exportable within the archive size budget.
        Archive::new(self.templates()?).validate()?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(t.id)
    }
    pub fn delete_template(&self, id: &str) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM templates WHERE id=?1", [id])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    pub fn import_templates(
        &self,
        archive: &Archive,
        conflict: Conflict,
    ) -> Result<ImportResult, String> {
        archive.validate()?;
        let tx = self
            .connection
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;
        let existing = self.templates()?;
        let mut result = ImportResult::default();
        for original in &archive.templates {
            let mut t = original.clone();
            if let Some(old) = existing.iter().find(|x| x.id == t.id) {
                match conflict {
                    Conflict::Skip => {
                        result.skipped += 1;
                        continue;
                    }
                    Conflict::KeepBoth => {
                        t.id = new_id(&tx)?;
                        result.added += 1;
                    }
                    Conflict::Replace => {
                        t.created_at = old.created_at;
                        t.updated_at = model::now().max(old.created_at);
                        result.replaced += 1;
                    }
                }
            } else {
                result.added += 1;
            }
            if existing.len() + result.added > MAX_TEMPLATES {
                return Err(tr!("Limite de modèles atteinte", "Template limit reached").into());
            }
            write(&tx, &t)?;
        }
        Archive::new(self.templates()?).validate()?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(result)
    }
}
