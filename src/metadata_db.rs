use rusqlite::Connection;

use crate::config::Config;
use crate::fs::FileMetadata;
use std::rc::Rc;

#[derive(Debug)]
pub struct MetadataDB {
    connection: Connection,
    config: Rc<Config>,
}

impl MetadataDB {
    pub fn new(config: Rc<Config>) -> Result<MetadataDB, rusqlite::Error> {
        let connection = Connection::open(config.database_path())?;
        let sql_string = std::include_str!("metadata_db.sql");
        connection.execute_batch(sql_string)?;
        Ok(MetadataDB { connection, config })
    }

    pub(crate) fn create(&self, meta: &FileMetadata) -> Result<i64, rusqlite::Error> {
        let query = r#"

INSERT INTO trash_entry (abspath, file_size, blake3sum, mtime, atime, unix_mode, uid, gid)
VALUES (:abspath, :file_size, :blake3sum, :mtime, :atime, :unix_mode, :uid, :gid)

"#;
        let rows_changed = self.connection.execute(
            query,
            &[
                (":abspath", &meta.abspath),
                (":file_size", &meta.file_size.to_string()),
                (":blake3sum", &meta.blake3sum),
                (":mtime", &meta.mtime.to_string()),
                (":atime", &meta.atime.to_string()),
                (":unix_mode", &meta.unix_mode.to_string()),
                (":uid", &meta.uid.to_string()),
                (":gid", &meta.gid.to_string()),
            ],
        )?;
        if rows_changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        let inserted_id = self.connection.last_insert_rowid();
        Ok(inserted_id)
    }

    pub(crate) fn delete(&self, trash_entry_id: i64) -> Result<(), rusqlite::Error> {
        let query = r#"
DELETE FROM trash_entry WHERE id = :id
        "#;
        let _ = self
            .connection
            .execute(query, &[(":id", &trash_entry_id)])?;
        Ok(())
    }

    pub(crate) fn find(
        &self,
        abspath: &std::path::Path,
    ) -> Result<Vec<(i64, FileMetadata)>, rusqlite::Error> {
        let query = r#"
SELECT id, abspath, file_size, blake3sum, mtime, atime, unix_mode, uid, gid
FROM trash_entry
WHERE abspath = :abspath
ORDER BY created_at DESC
        "#;
        let mut stmt = self.connection.prepare(query)?;
        let rows = stmt.query_map(&[(":abspath", &abspath.to_string_lossy())], |row| {
            Ok((
                row.get("id")?,
                FileMetadata {
                    abspath: row.get("abspath")?,
                    file_size: row.get("file_size")?,
                    blake3sum: row.get("blake3sum")?,
                    mtime: row.get("mtime")?,
                    atime: row.get("atime")?,
                    unix_mode: row.get("unix_mode")?,
                    uid: row.get("uid")?,
                    gid: row.get("gid")?,
                },
            ))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub(crate) fn find_by_id(&self, id: i64) -> Result<Option<FileMetadata>, rusqlite::Error> {
        let query = r#"
SELECT abspath, file_size, blake3sum, mtime, atime, unix_mode, uid, gid
FROM trash_entry
WHERE id = :id
        "#;
        let mut stmt = self.connection.prepare(query)?;
        let mut rows = stmt.query_map(&[(":id", &id)], |row| {
            Ok(Some(FileMetadata {
                abspath: row.get("abspath")?,
                file_size: row.get("file_size")?,
                blake3sum: row.get("blake3sum")?,
                mtime: row.get("mtime")?,
                atime: row.get("atime")?,
                unix_mode: row.get("unix_mode")?,
                uid: row.get("uid")?,
                gid: row.get("gid")?,
            }))
        })?;
        let result = rows.next().unwrap()?;
        Ok(result)
    }
}
