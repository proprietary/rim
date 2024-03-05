use clap::Parser;
use rim::{config::Config, App};
use std::{path::PathBuf, process::Command, rc::Rc};

#[derive(Parser, Debug)]
#[clap(
    name = "rim",
    version = "0.1.0",
    author = "Zelly Snyder",
    about = "Recycle bin for the command line"
)]
struct Opts {
    #[arg(help = "File or directory to recycle")]
    filename: PathBuf,

    #[arg(short, long)]
    verbose: bool,

    #[arg(
        short,
        long,
        default_value = "false",
        help = "Recursively recycle directories"
    )]
    recursive: bool,

    #[arg(long)]
    recover: bool,

    #[arg(short, long)]
    config: Option<PathBuf>,
}

fn main() {
    let opts: Opts = Opts::parse();
    let config = Rc::new(Config::load(opts.config.clone()).expect("Error opening config file"));
    let app = App::new(config).unwrap();
    if opts.recover {
        let p = subcommand_path(&ExternalSubcommand::Recover).expect("Could not find rim-recover");
        let mut cmd = Command::new(p);
        cmd.arg(opts.filename.clone());
        if opts.verbose {
            cmd.arg("--verbose");
        }
        if opts.recursive {
            cmd.arg("--recursive");
        }
        if let Some(config) = &opts.config {
            cmd.arg("--config");
            cmd.arg(config);
        }
    }
    let mut filename = opts.filename.clone();
    if filename.is_relative() {
        let cwd = std::env::current_dir().expect("Can't tell what directory this is in");
        filename = cwd.join(&filename);
    }
    if opts.recursive {
        app.recycle_subtree(&filename).unwrap();
    } else if filename.is_dir() {
        app.recycle_dir(&filename).unwrap();
    } else {
        app.recycle_file(&filename).unwrap();
    }
}

enum ExternalSubcommand {
    Recover,
    Wrap,
}

impl std::string::ToString for ExternalSubcommand {
    fn to_string(&self) -> String {
        use ExternalSubcommand::*;
        match self {
            Recover => "recover".to_string(),
            Wrap => "wrap".to_string(),
        }
    }
}

impl std::str::FromStr for ExternalSubcommand {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ExternalSubcommand::*;
        match s {
            "recover" => Ok(Recover),
            "wrap" => Ok(Wrap),
            _ => Err(format!("Unknown subcommand: {}", s)),
        }
    }
}

/// Find the path to a related CLI binary which has the prefix "rim-".
fn subcommand_path(subcommand: &ExternalSubcommand) -> Result<PathBuf, std::io::Error> {
    let mut here = std::env::current_exe()?;
    here.pop();
    let p = "rim-".to_string() + &subcommand.to_string();
    here.push(p);
    Ok(here)
}
