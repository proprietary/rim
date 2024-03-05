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

    pub fn load(config_file: Option<PathBuf>) -> Result<Config, std::io::Error> {
        match config_file {
            Some(path) => Config::open(&path),
            None => {
                let mut config = Config::default();
                let mut config_paths: Vec<PathBuf> = vec![];
                if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
                    config_paths.push(
                        PathBuf::from(xdg_config_home)
                            .join("rim")
                            .join("config.yaml"),
                    );
                }
                config_paths.push(
                    dirs_next::home_dir()
                        .unwrap()
                        .join(".config")
                        .join("rim")
                        .join("config.yaml"),
                );
                config_paths.push(
                    dirs_next::home_dir()
                        .unwrap()
                        .join(".rim")
                        .join("config.yaml"),
                );
                config_paths.push(dirs_next::home_dir().unwrap().join(".rim.yaml"));
                let mut any_found: bool = false;
                for path in config_paths.iter() {
                    if path.exists() {
                        config = Config::open(path).expect("Error opening config file");
                        any_found = true;
                        break;
                    }
                }
                if !any_found {
                    let destination = Self::create()?;
                    config.save(&destination)?;
                }
                Ok(config)
            }
        }
    }

    /// Creates config directory, returning the path to the config
    /// file that should be written to.
    fn create() -> Result<PathBuf, std::io::Error> {
        let mut destination = dirs_next::home_dir().ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory; only Unix-like operating systems are supported",
        ))?;
        if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
            destination = destination.join(xdg_config_home);
            destination = destination.join("rim");
        } else {
            destination = destination.join(".rim");
        }
        std::fs::create_dir_all(&destination)?;
        destination = destination.join("rim.yaml");
        Ok(destination)
    }
}
