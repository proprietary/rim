use clap::Parser;
use rim::{config::Config, App};
use std::rc::Rc;

#[derive(Parser, Debug)]
#[clap(
    name = "rim",
    version = "0.1.0",
    author = "Zelly Snyder",
    about = "Recycle bin for the command line"
)]
struct Opts {
    filename: String,

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
        None => Config::default(),
    });
    let app = App::new(config).unwrap();
    let filename = std::path::PathBuf::from(&opts.filename);
    if opts.recover {
        // TODO: display TUI for selecting file to recover
        // app.recover_file(&filename).unwrap();
    } else {
        app.recycle_file(&filename).unwrap();
    }
}
