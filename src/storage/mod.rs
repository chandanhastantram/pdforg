//! pdf-storage — SQLite persistence layer for documents, versions, comments, preferences.

use rusqlite::{Connection, params, OptionalExtension};
use serde::{Serialize, de::DeserializeOwned};
use serde_json;
use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use thiserror::Error;
use crate::core::document::{Document, DocumentSnapshot, Comment};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("ZIP error: {0}")]
    Zip(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecentEntry {
    pub id: Uuid,
    pub title: String,
    pub path: Option<String>,
    pub opened_at: chrono::DateTime<chrono::Utc>,
    pub doc_type: DocType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DocType {
    Writer,
    Sheets,
    Slides,
    Pdf,
}

/// The central storage layer — all persistence goes through this struct
pub struct Store {
    data_dir: PathBuf,
    app_db: Connection,
}

impl Store {
    /// Open (or create) the store at the given data directory
    pub fn open(data_dir: &Path) -> Result<Self, StorageError> {
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("app.db");
        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrent performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        let store = Store { data_dir: data_dir.to_path_buf(), app_db: conn };
        store.initialize_schema()?;
        Ok(store)
    }

    fn initialize_schema(&self) -> Result<(), StorageError> {
        self.app_db.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                doc_type TEXT NOT NULL DEFAULT 'writer',
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS versions (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                content TEXT NOT NULL,
                label TEXT NOT NULL,
                created_at TEXT NOT NULL,
                created_by TEXT NOT NULL,
                FOREIGN KEY(document_id) REFERENCES documents(id)
            );

            CREATE TABLE IF NOT EXISTS comments (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL,
                content TEXT NOT NULL,
                author TEXT NOT NULL,
                created_at TEXT NOT NULL,
                resolved INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY(document_id) REFERENCES documents(id)
            );

            CREATE TABLE IF NOT EXISTS preferences (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS recent_docs (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                path TEXT,
                opened_at TEXT NOT NULL,
                doc_type TEXT NOT NULL DEFAULT 'writer'
            );

            CREATE INDEX IF NOT EXISTS idx_versions_doc ON versions(document_id);
            CREATE INDEX IF NOT EXISTS idx_comments_doc ON comments(document_id);
            CREATE INDEX IF NOT EXISTS idx_recent_time ON recent_docs(opened_at DESC);
        "#)?;
        Ok(())
    }

    // ─── Document CRUD ───────────────────────────────────────────────────────

    pub fn save_document(&mut self, doc: &Document) -> Result<(), StorageError> {
        let content = serde_json::to_string(doc)?;
        let now = chrono::Utc::now().to_rfc3339();

        self.app_db.execute(
            "INSERT OR REPLACE INTO documents (id, title, content, doc_type, created_at, modified_at)
             VALUES (?1, ?2, ?3, 'writer', COALESCE((SELECT created_at FROM documents WHERE id=?1), ?4), ?4)",
            params![doc.id.to_string(), doc.title, content, now],
        )?;

        // Update recent
        self.app_db.execute(
            "INSERT OR REPLACE INTO recent_docs (id, title, opened_at, doc_type) VALUES (?1, ?2, ?3, 'writer')",
            params![doc.id.to_string(), doc.title, now],
        )?;

        Ok(())
    }

    pub fn load_document(&self, id: Uuid) -> Result<Document, StorageError> {
        let content: String = self.app_db.query_row(
            "SELECT content FROM documents WHERE id = ?1",
            params![id.to_string()],
            |row| row.get(0),
        ).optional()?.ok_or_else(|| StorageError::NotFound(id.to_string()))?;

        Ok(serde_json::from_str(&content)?)
    }

    pub fn delete_document(&self, id: Uuid) -> Result<(), StorageError> {
        self.app_db.execute("DELETE FROM documents WHERE id = ?1", params![id.to_string()])?;
        self.app_db.execute("DELETE FROM versions WHERE document_id = ?1", params![id.to_string()])?;
        self.app_db.execute("DELETE FROM comments WHERE document_id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn list_documents(&self) -> Result<Vec<RecentEntry>, StorageError> {
        self.list_recent(100)
    }

    // ─── Recent documents ───────────────────────────────────────────────────

    pub fn list_recent(&self, limit: usize) -> Result<Vec<RecentEntry>, StorageError> {
        let mut stmt = self.app_db.prepare(
            "SELECT id, title, path, opened_at, doc_type FROM recent_docs ORDER BY opened_at DESC LIMIT ?1"
        )?;

        let entries = stmt.query_map(params![limit as i64], |row| {
            let id_str: String = row.get(0)?;
            let opened_str: String = row.get(3)?;
            let type_str: String = row.get(4)?;
            Ok((id_str, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?, opened_str, type_str))
        })?
        .filter_map(|r| r.ok())
        .filter_map(|(id_str, title, path, opened_str, type_str)| {
            let id = Uuid::parse_str(&id_str).ok()?;
            let opened_at = chrono::DateTime::parse_from_rfc3339(&opened_str).ok()?.with_timezone(&chrono::Utc);
            let doc_type = match type_str.as_str() {
                "sheets" => DocType::Sheets,
                "slides" => DocType::Slides,
                "pdf" => DocType::Pdf,
                _ => DocType::Writer,
            };
            Some(RecentEntry { id, title, path, opened_at, doc_type })
        })
        .collect();

        Ok(entries)
    }

    // ─── Versions ────────────────────────────────────────────────────────────

    pub fn save_version(&mut self, snap: &DocumentSnapshot) -> Result<(), StorageError> {
        let content = serde_json::to_string(&snap.document)?;
        self.app_db.execute(
            "INSERT INTO versions (id, document_id, content, label, created_at, created_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                snap.id.to_string(),
                snap.document_id.to_string(),
                content,
                snap.version_label,
                snap.created_at.to_rfc3339(),
                snap.created_by,
            ],
        )?;
        Ok(())
    }

    pub fn list_versions(&self, doc_id: Uuid) -> Result<Vec<VersionMeta>, StorageError> {
        let mut stmt = self.app_db.prepare(
            "SELECT id, label, created_at, created_by FROM versions WHERE document_id = ?1 ORDER BY created_at DESC"
        )?;
        let metas = stmt.query_map(params![doc_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .filter_map(|(id_str, label, at_str, author)| {
            let id = Uuid::parse_str(&id_str).ok()?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&at_str).ok()?.with_timezone(&chrono::Utc);
            Some(VersionMeta { id, document_id: doc_id, label, created_at, created_by: author })
        })
        .collect();
        Ok(metas)
    }

    pub fn load_version(&self, version_id: Uuid) -> Result<DocumentSnapshot, StorageError> {
        let (doc_id_str, content, label, at_str, author): (String, String, String, String, String) =
            self.app_db.query_row(
                "SELECT document_id, content, label, created_at, created_by FROM versions WHERE id = ?1",
                params![version_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            ).optional()?.ok_or_else(|| StorageError::NotFound(version_id.to_string()))?;

        let document: Document = serde_json::from_str(&content)?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&at_str)
            .map_err(|e| StorageError::NotFound(e.to_string()))?
            .with_timezone(&chrono::Utc);

        Ok(DocumentSnapshot {
            id: version_id,
            document_id: Uuid::parse_str(&doc_id_str)
                .map_err(|e| StorageError::NotFound(e.to_string()))?,
            version_label: label,
            created_at,
            created_by: author,
            document,
        })
    }

    // ─── Comments ────────────────────────────────────────────────────────────

    pub fn save_comment(&mut self, doc_id: Uuid, comment: &Comment) -> Result<(), StorageError> {
        let content = serde_json::to_string(comment)?;
        self.app_db.execute(
            "INSERT OR REPLACE INTO comments (id, document_id, content, author, created_at, resolved)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                comment.id.to_string(),
                doc_id.to_string(),
                content,
                comment.author,
                comment.created_at.to_rfc3339(),
                comment.resolved as i64,
            ],
        )?;
        Ok(())
    }

    pub fn list_comments(&self, doc_id: Uuid) -> Result<Vec<Comment>, StorageError> {
        let mut stmt = self.app_db.prepare(
            "SELECT content FROM comments WHERE document_id = ?1 ORDER BY created_at ASC"
        )?;
        let comments = stmt.query_map(params![doc_id.to_string()], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .filter_map(|s| serde_json::from_str(&s).ok())
            .collect();
        Ok(comments)
    }

    // ─── Preferences ─────────────────────────────────────────────────────────

    pub fn get_pref<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, StorageError> {
        let value: Option<String> = self.app_db.query_row(
            "SELECT value FROM preferences WHERE key = ?1",
            params![key],
            |row| row.get(0),
        ).optional()?;

        match value {
            None => Ok(None),
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
        }
    }

    pub fn set_pref<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), StorageError> {
        let json = serde_json::to_string(value)?;
        self.app_db.execute(
            "INSERT OR REPLACE INTO preferences (key, value) VALUES (?1, ?2)",
            params![key, json],
        )?;
        Ok(())
    }

    pub fn delete_pref(&self, key: &str) -> Result<(), StorageError> {
        self.app_db.execute("DELETE FROM preferences WHERE key = ?1", params![key])?;
        Ok(())
    }

    // ─── .pdfo file format (ZIP + JSON) ───────────────────────────────────────

    /// Export document to .pdfo format (ZIP containing content.json + media/)
    pub fn export_pdfo(&self, doc: &Document) -> Result<Vec<u8>, StorageError> {
        use std::io::{Write, Cursor};
        use zip::{ZipWriter, write::FileOptions};

        let buf = Vec::new();
        let mut cursor = Cursor::new(buf);
        let mut zip = ZipWriter::new(&mut cursor);
        let opts = FileOptions::<'_, ()>::default().compression_method(zip::CompressionMethod::Deflated);

        // content.json
        zip.start_file("content.json", opts).map_err(|e| StorageError::Zip(e.to_string()))?;
        let json = serde_json::to_string_pretty(doc)?;
        zip.write_all(json.as_bytes())?;

        // meta.json
        zip.start_file("meta.json", opts).map_err(|e| StorageError::Zip(e.to_string()))?;
        let meta = serde_json::json!({
            "format": "pdf",
            "version": "0.1.0",
            "created": chrono::Utc::now().to_rfc3339(),
        });
        zip.write_all(meta.to_string().as_bytes())?;

        zip.finish().map_err(|e| StorageError::Zip(e.to_string()))?;
        Ok(cursor.into_inner())
    }

    /// Import document from .pdfo format
    pub fn import_pdfo(bytes: &[u8]) -> Result<Document, StorageError> {
        use std::io::{Read, Cursor};
        use zip::ZipArchive;

        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)
            .map_err(|e| StorageError::Zip(e.to_string()))?;

        let mut entry = archive.by_name("content.json")
            .map_err(|e| StorageError::Zip(e.to_string()))?;
        let mut content = String::new();
        entry.read_to_string(&mut content)?;
        Ok(serde_json::from_str(&content)?)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionMeta {
    pub id: Uuid,
    pub document_id: Uuid,
    pub label: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_open_and_prefs() {
        let dir = std::env::temp_dir().join("pdf_test_store");
        let mut store = Store::open(&dir).unwrap();
        store.set_pref("theme", &"dark").unwrap();
        let theme: Option<String> = store.get_pref("theme").unwrap();
        assert_eq!(theme, Some("dark".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_document_save_load() {
        let dir = std::env::temp_dir().join("pdf_test_docs");
        let mut store = Store::open(&dir).unwrap();
        let doc = Document::default();
        let id = doc.id;
        store.save_document(&doc).unwrap();
        let loaded = store.load_document(id).unwrap();
        assert_eq!(loaded.id, id);
        std::fs::remove_dir_all(&dir).ok();
    }
}
