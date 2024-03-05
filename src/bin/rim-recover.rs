use clap::Parser;
use rim::{config::Config, App};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    name = "rim-recover",
    version = "0.1.0",
    author = "Zelly Snyder",
    about = "Recycle bin for the command line"
)]
struct Opts {
    filename: Option<String>,

    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() {
    let opts: Opts = Opts::parse();
    let config = std::rc::Rc::new(Config::load(opts.config).expect("Error opening config file"));
    let app = App::new(config).unwrap();
    let id = opts.filename.unwrap().parse::<i64>().unwrap();
    app.recover_file(id).unwrap();
}
