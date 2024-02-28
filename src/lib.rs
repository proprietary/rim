pub mod config;
mod fs;
pub mod metadata_db;
use std::rc::Rc;

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
        let trash_filename = self.generate_trash_path(meta.abspath.as_ref(), id);
        std::fs::rename(trash_filename, meta.abspath)
    }

    fn generate_trash_path(&self, path: &std::path::Path, id: i64) -> std::path::PathBuf {
        let mut trash_path = self.config.trashdir.clone();
        trash_path.push(path.file_name().unwrap());
        trash_path.push("_");
        trash_path.push(id.to_string());
        trash_path
    }

    fn id_from_trash_path(&self, path: &std::path::Path) -> Result<i64, std::io::Error> {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let id_str = filename.split('_').last().unwrap();
        let id = id_str
            .parse::<i64>()
            .expect("Invalid trash filename: Should have an integer id at the end of the filename");
        Ok(id)
    }
}