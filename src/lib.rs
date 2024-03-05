pub mod config;
mod fs;
pub mod metadata_db;
use std::{
    os::unix::fs::{chown, PermissionsExt},
    path::PathBuf,
    rc::Rc,
};

pub struct App {
    pub config: Rc<config::Config>,
    metadata_db: metadata_db::MetadataDB,
}

impl App {
    pub fn new(config: Rc<config::Config>) -> Result<App, rusqlite::Error> {
        let metadata_db = metadata_db::MetadataDB::new(config.clone())?;
        Ok(App {
            config,
            metadata_db,
        })
    }

    pub fn recycle_subtree(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        todo!()
    }

    pub fn recycle_dir(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        todo!()
    }

    pub fn recycle_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let meta = fs::read_file_meta(path)?;
        let id = match self.metadata_db.create(&meta) {
            Ok(id) => id,
            Err(e) => {
                println!("Error creating metadata entry: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error creating metadata entry",
                ));
            }
        };
        let trash_path = self.generate_trash_path(path, id);
        match std::fs::rename(path, trash_path) {
            Ok(_) => (),
            Err(e) => {
                println!("Error moving file to trash: {}", e);
                let _ = self.metadata_db.delete(id);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error moving file to trash",
                ));
            }
        }
        Ok(())
    }

    pub fn recover_file(&self, id: i64) -> Result<(), std::io::Error> {
        let meta = match self.metadata_db.find_by_id(id) {
            Ok(Some(meta)) => meta,
            Ok(None) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found in trash",
                ));
            }
            Err(e) => {
                println!("Error finding metadata entry: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error finding metadata entry",
                ));
            }
        };
        let original_path: std::path::PathBuf = meta.abspath.into();
        let trash_filename = self.generate_trash_path(&original_path, id);
        if original_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "File already exists",
            ));
        }
        std::fs::rename(trash_filename, &original_path)?;
        let perms: std::fs::Permissions = std::fs::Permissions::from_mode(meta.unix_mode);
        std::fs::set_permissions(&original_path, perms)?;
        chown(original_path, Some(meta.uid), Some(meta.gid))?;
        Ok(())
    }

    fn generate_trash_path(&self, path: &std::path::Path, id: i64) -> std::path::PathBuf {
        let mut trash_path = self.config.trashdir.clone();
        trash_path.push(path.file_name().unwrap());
        trash_path.push("_");
        trash_path.push(id.to_string());
        trash_path
    }

    #[allow(dead_code)]
    fn id_from_trash_path(&self, path: &std::path::Path) -> Result<i64, std::io::Error> {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let id_str = filename.split('_').last().unwrap();
        let id = id_str
            .parse::<i64>()
            .expect("Invalid trash filename: Should have an integer id at the end of the filename");
        Ok(id)
    }

    pub fn list_recent(
        &self,
        n: u32,
    ) -> Result<Vec<(i64, crate::fs::FileMetadata)>, std::io::Error> {
        let results = self.metadata_db.recent(n).expect("SQL error");
        Ok(results)
    }

    /// Runs a maintenance task which permanently deletes the expired files.
    pub fn run_maintenance(&self) -> Result<(), std::io::Error> {
        let now: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expired = match self.metadata_db.find_expired(now) {
            Ok(x) => x,
            Err(rusqlite::Error::SqlInputError {
                error,
                msg,
                sql,
                offset,
            }) => {
                eprintln!(
                    "SQL error: {}, error={}, sql={}, offset={}",
                    msg, error, sql, offset
                );
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "SQL Error"));
            }
            Err(e) => {
                eprintln!("SQL error: {}", e);
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "SQL Error"));
            }
        };
        let realpaths: Vec<PathBuf> = expired
            .into_iter()
            .map(|(id, meta)| {
                let p: PathBuf = PathBuf::from(&meta.abspath);
                self.generate_trash_path(&p, id)
            })
            .collect();
        let realpaths = metadata_db::toposort_files(&realpaths);
        for realpath in realpaths.iter() {
            if realpath.is_dir() {
                std::fs::remove_dir(realpath)?;
            } else {
                std::fs::remove_file(realpath)?;
            }
        }
        Ok(())
    }
}
