use rusqlite::{Connection, OpenFlags};

use crate::config::Config;
use crate::fs::FileMetadata;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug)]
pub struct TrashEntry {
    pub id: i64,
    pub metadata: FileMetadata,
    pub trash_path: PathBuf,
}

#[derive(Debug)]
pub struct MetadataDB {
    connection: Connection,
    config: Rc<Config>,
}

impl MetadataDB {
    pub fn new(config: Rc<Config>) -> Result<MetadataDB, rusqlite::Error> {
        let connection = Connection::open_with_flags(
            config.database_path(),
            OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE,
        )?;
        let sql_string = std::include_str!("metadata_db.sql");
        connection.execute_batch(sql_string)?;
        Ok(MetadataDB { connection, config })
    }

    pub(crate) fn recent(&self, n: u32) -> Result<Vec<TrashEntry>, rusqlite::Error> {
        let query = r#"
SELECT
    id,
    original_path,
    trash_path,
    file_size,
    blake3sum,
    mtime,
    atime,
    unix_mode,
    uid,
    gid
FROM
    trash_entry
ORDER BY
    created_at DESC
LIMIT :n
        "#;
        let mut stmt = self.connection.prepare(query)?;
        let rows = stmt.query_map(&[(":n", &n.to_string())], |row| {
            let id: i64 = row.get("id")?;
            let metadata = FileMetadata {
                original_path: row.get("original_path")?,
                file_size: row.get("file_size")?,
                blake3sum: row.get("blake3sum")?,
                mtime: row.get("mtime")?,
                atime: row.get("atime")?,
                unix_mode: row.get("unix_mode")?,
                uid: row.get("uid")?,
                gid: row.get("gid")?,
            };
            let trash_path: PathBuf = row.get::<_, String>("trash_path")?.into();
            Ok(TrashEntry {
                id,
                metadata,
                trash_path,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub(crate) fn create(
        &self,
        meta: FileMetadata,
        generated_path: &Path,
    ) -> Result<TrashEntry, rusqlite::Error> {
        let query = r#"
INSERT INTO
    trash_entry (
        original_path,
        trash_path,
        file_size,
        blake3sum,
        mtime,
        atime,
        unix_mode,
        uid,
        gid,
        expiration
    )
VALUES
    (
        :original_path,
        :trash_path,
        :file_size,
        :blake3sum,
        :mtime,
        :atime,
        :unix_mode,
        :uid,
        :gid,
        :expiration
    )
"#;
        let expiration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + self.config.ttl;
        let rows_changed = self.connection.execute(
            query,
            &[
                (":original_path", &meta.original_path),
                (":trash_path", &generated_path.to_string_lossy().to_string()),
                (":file_size", &meta.file_size.to_string()),
                (":blake3sum", &meta.blake3sum),
                (":mtime", &meta.mtime.to_string()),
                (":atime", &meta.atime.to_string()),
                (":unix_mode", &meta.unix_mode.to_string()),
                (":uid", &meta.uid.to_string()),
                (":gid", &meta.gid.to_string()),
                (":expiration", &expiration.to_string()),
            ],
        )?;
        if rows_changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        let inserted_id = self.connection.last_insert_rowid();
        Ok(TrashEntry {
            metadata: meta,
            trash_path: generated_path.into(),
            id: inserted_id,
        })
    }

    pub(crate) fn delete(&self, trash_entry_id: i64) -> Result<(), rusqlite::Error> {
        let query = r#"
DELETE FROM
    trash_entry
WHERE
    id = :id
"#;
        let _ = self
            .connection
            .execute(query, &[(":id", &trash_entry_id)])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn find(
        &self,
        abspath: &std::path::Path,
    ) -> Result<Vec<TrashEntry>, rusqlite::Error> {
        let query = r#"
SELECT
    id,
    original_path,
    trash_path,
    file_size,
    blake3sum,
    mtime,
    atime,
    unix_mode,
    uid,
    gid
FROM
    trash_entry
WHERE
    original_path = :original_path
ORDER BY
    created_at DESC
        "#;
        let mut stmt = self.connection.prepare(query)?;
        let rows = stmt.query_map(&[(":abspath", &abspath.to_string_lossy())], |row| {
            Ok(TrashEntry {
                id: row.get("id")?,
                metadata: FileMetadata {
                    original_path: row.get("original_path")?,
                    file_size: row.get("file_size")?,
                    blake3sum: row.get("blake3sum")?,
                    mtime: row.get("mtime")?,
                    atime: row.get("atime")?,
                    unix_mode: row.get("unix_mode")?,
                    uid: row.get("uid")?,
                    gid: row.get("gid")?,
                },
                trash_path: row.get::<_, String>("trash_path")?.into(),
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub(crate) fn find_by_id(&self, id: i64) -> Result<Option<TrashEntry>, rusqlite::Error> {
        let query = r#"
SELECT
    id,
    original_path,
    trash_path,
    file_size,
    blake3sum,
    mtime,
    atime,
    unix_mode,
    uid,
    gid
FROM
    trash_entry
WHERE
    id = :id
"#;
        let mut stmt = self.connection.prepare(query)?;
        let mut r = stmt.query_map(&[(":id", &id)], |row| {
            Ok(TrashEntry {
                id: row.get("id")?,
                metadata: FileMetadata {
                    original_path: row.get("original_path")?,
                    file_size: row.get("file_size")?,
                    blake3sum: row.get("blake3sum")?,
                    mtime: row.get("mtime")?,
                    atime: row.get("atime")?,
                    unix_mode: row.get("unix_mode")?,
                    uid: row.get("uid")?,
                    gid: row.get("gid")?,
                },
                trash_path: row.get::<_, String>("trash_path")?.into(),
            })
        })?;
        match r.next() {
            Some(Ok(meta)) => Ok(Some(meta)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    pub(crate) fn find_expired(&self, now: u64) -> Result<Vec<TrashEntry>, rusqlite::Error> {
        let query = r#"
SELECT
    id,
    original_path,
    trash_path,
    file_size,
    blake3sum,
    mtime,
    atime,
    unix_mode,
    uid,
    gid
FROM
    trash_entry
WHERE
    expiration < :now
ORDER BY
    abspath DESC
        "#;
        let mut stmt = self.connection.prepare(query)?;
        let rows = stmt.query_map(&[(":now", &now)], |row| {
            Ok(TrashEntry {
                id: row.get("id")?,
                metadata: FileMetadata {
                    original_path: row.get("original_path")?,
                    file_size: row.get("file_size")?,
                    blake3sum: row.get("blake3sum")?,
                    mtime: row.get("mtime")?,
                    atime: row.get("atime")?,
                    unix_mode: row.get("unix_mode")?,
                    uid: row.get("uid")?,
                    gid: row.get("gid")?,
                },
                trash_path: row.get::<_, String>("trash_path")?.into(),
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}

pub fn toposort_files(files: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut graph: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    for file in files {
        let path = file.clone();
        for ancestor in path.ancestors().skip(1) {
            if ancestor.has_root() && ancestor.components().count() == 1 {
                continue;
            }
            let child_name = path.clone();
            graph
                .entry(ancestor.to_path_buf())
                .or_default()
                .push(child_name);
        }
    }
    topological_sort(&graph)
}

fn topological_sort(graph: &HashMap<PathBuf, Vec<PathBuf>>) -> Vec<PathBuf> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut sorted_paths: Vec<PathBuf> = Vec::new();

    fn dfs(
        node: &PathBuf,
        graph: &HashMap<PathBuf, Vec<PathBuf>>,
        visited: &mut HashSet<PathBuf>,
        sorted_paths: &mut Vec<PathBuf>,
    ) {
        if visited.contains(node) {
            return;
        }
        visited.insert(node.to_path_buf());
        for child in graph.get(node).unwrap_or(&Vec::new()) {
            dfs(child, graph, visited, sorted_paths);
        }
        sorted_paths.push(node.to_path_buf());
    }

    for node in graph.keys() {
        dfs(node, graph, &mut visited, &mut sorted_paths);
    }
    sorted_paths
}

#[cfg(test)]
mod test {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> MetadataDB {
        let connection = Connection::open_in_memory().unwrap();
        let sql_string = std::include_str!("metadata_db.sql");
        connection.execute_batch(sql_string).unwrap();
        let config = Rc::new(Config::default());
        MetadataDB { connection, config }
    }

    #[test]
    fn test_create() {
        let suite = setup();
        let meta = FileMetadata {
            original_path: "/tmp/testfile".to_string(),
            file_size: 1234,
            blake3sum: "1234567890abcdef".to_string(),
            mtime: 123456,
            atime: 123456,
            unix_mode: 0o644,
            uid: 1000,
            gid: 1000,
        };
        let generated_path = PathBuf::from("/tmp/Some/Generated/Path");
        let entry = suite.create(meta.clone(), &generated_path).unwrap();
        assert_eq!(entry.id, 1);
        assert_eq!(meta.original_path, entry.metadata.original_path);
        assert_eq!(meta.file_size, entry.metadata.file_size);
        assert_eq!(meta.blake3sum, entry.metadata.blake3sum);
        assert_eq!(meta.mtime, entry.metadata.mtime);
        assert_eq!(meta.atime, entry.metadata.atime);
        assert_eq!(meta.unix_mode, entry.metadata.unix_mode);
        assert_eq!(meta.uid, entry.metadata.uid);
        assert_eq!(meta.gid, entry.metadata.gid);
    }

    #[test]
    fn test_delete() {
        let suite = setup();
        let meta = FileMetadata {
            original_path: "/tmp/testfile".to_string(),
            file_size: 1234,
            blake3sum: "1234567890abcdef".to_string(),
            mtime: 123456,
            atime: 123456,
            unix_mode: 0o644,
            uid: 1000,
            gid: 1000,
        };
        let generated_path = PathBuf::from("/tmp/Some/Generated/Path");
        let entry = suite.create(meta.clone(), &generated_path).unwrap();
        suite.delete(entry.id).unwrap();
        let result = suite.find_by_id(entry.id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_insert_and_find() {
        let suite = setup();
        let meta = FileMetadata {
            original_path: "/tmp/testfile".to_string(),
            file_size: 1234,
            blake3sum: "cafebabe".to_string(),
            mtime: 1709096470,
            atime: 1709096477,
            unix_mode: 0o755,
            uid: 1000,
            gid: 1000,
        };
        let generated_path = PathBuf::from("/tmp/a.txt");
        let entry = suite.create(meta.clone(), &generated_path).unwrap();
        let meta_found = suite.find_by_id(entry.id).unwrap().unwrap();
        assert_eq!(meta.file_size, meta_found.metadata.file_size);
        assert_eq!(meta.blake3sum, meta_found.metadata.blake3sum);
        assert_eq!(meta.mtime, meta_found.metadata.mtime);
        assert_eq!(meta.atime, meta_found.metadata.atime);
        assert_eq!(meta.unix_mode, meta_found.metadata.unix_mode);
        assert_eq!(meta.uid, meta_found.metadata.uid);
        assert_eq!(meta.gid, meta_found.metadata.gid);
        assert_eq!(meta.original_path, meta_found.metadata.original_path);
    }

    #[test]
    fn test_toposort_files() {
        let mut files = vec![
            PathBuf::from("/tmp"),
            PathBuf::from("/tmp/foo/bar/baz/quux"),
            PathBuf::from("/tmp/foo"),
            PathBuf::from("/tmp/foo/bar/baz/qux"),
            PathBuf::from("/tmp/foo/bar/baz"),
            PathBuf::from("/tmp/foo/bar"),
        ];
        let sorted = toposort_files(&files);
        let expected = vec![
            PathBuf::from("/tmp/foo/bar/baz/quux"),
            PathBuf::from("/tmp/foo/bar/baz/qux"),
            PathBuf::from("/tmp/foo/bar/baz"),
            PathBuf::from("/tmp/foo/bar"),
            PathBuf::from("/tmp/foo"),
            PathBuf::from("/tmp"),
        ];
        assert_eq!(sorted, expected);
    }
}
