use clap::Parser;
use std::fs::{self, ReadDir};

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long, default_value = ".")]
    path: String,

    #[clap(short, long, num_args(0..))]
    extensions: Vec<String>,
    
    #[clap(short, long)]
    dry_run: bool,
    
    #[clap(short, long)]
    backup: bool,
}

fn get_dir_entries(entries: ReadDir) {
    for entry in entries {
        if let Ok(_entry) = entry {
            println!("{:?}", _entry.path());
        }
    }
}

fn main() {
    let args = Cli::parse();
    let directory = fs::read_dir(args.path);

    match directory {
        Ok(_dir) => {
            get_dir_entries(_dir);
        }
        Err(e) => { println!("Error opening directory: {}", e); }
    }
}
