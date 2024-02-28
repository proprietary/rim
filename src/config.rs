//! Configuration file format

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_DATABASE_NAME: &str = "rim.db";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub trashdir: PathBuf,
    pub database_name: String,
    pub ttl: u64,
}

impl Default for Config {
    fn default() -> Config {
        let tempdir = std::env::temp_dir();
        let trashdir = tempdir.join("rim");
        Config {
            trashdir,
            database_name: DEFAULT_DATABASE_NAME.to_string(),
            ttl: 604800,
        }
    }
}

impl Config {
    pub fn save(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let s = serde_yaml::to_string(self).unwrap();
        std::fs::write(path, s)
    }

    pub fn open(path: &std::path::Path) -> Result<Config, std::io::Error> {
        let s = std::fs::read_to_string(path)?;
        let config: Config = match serde_yaml::from_str(&s) {
            Ok(c) => c,
            Err(e) => {
                println!("Error parsing config file: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Error parsing config file",
                ));
            }
        };
        Ok(config)
    }

    pub fn database_path(&self) -> PathBuf {
        self.trashdir.join(&self.database_name)
    }
}
