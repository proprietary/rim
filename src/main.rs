use clap::Parser;
use rim::{config::Config, App};
use std::{path::PathBuf, rc::Rc};

#[derive(Parser, Debug)]
#[clap(
    name = "rim",
    version = "0.1.0",
    author = "Zelly Snyder",
    about = "Recycle bin for the command line"
)]
struct Opts {
    filename: Option<String>,

    #[arg(short, long)]
    recover: bool,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    config: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();
    println!("{:?}", opts);
    let config = Rc::new(match opts.config {
        Some(path) => {
            let path = std::path::PathBuf::from(path);
            Config::open(&path).unwrap()
        }
        None => {
            let mut config = Config::default();
            let config_paths: Vec<PathBuf> = vec![
                dirs_next::home_dir()
                    .unwrap()
                    .join("..config")
                    .join("rim")
                    .join("config.yaml"),
                dirs_next::home_dir()
                    .unwrap()
                    .join(".rim")
                    .join("config.yaml"),
                dirs_next::home_dir().unwrap().join(".rim.yaml"),
            ];
            for path in config_paths {
                if path.exists() {
                    config = Config::open(&path).expect("Error opening config file");
                }
            }
            config
        }
    });
    let app = App::new(config).unwrap();
    match opts.filename {
        Some(filename) => {
            let mut filename = std::path::PathBuf::from(&filename);
            if filename.is_relative() {
                let cwd = std::env::current_dir().expect("Can't tell what directory this is in");
                filename = cwd.join(filename);
            }
            if opts.recover {
                // TODO: display TUI for selecting file to recover
                // app.recover_file(&filename).unwrap();
                todo!();
            } else {
                app.recycle_file(&filename).unwrap();
            }
        }
        None => {
            todo!();
        }
    }
}
