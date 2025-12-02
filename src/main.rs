use clap::Parser;
use std::fs::{self, ReadDir};
use serde_json::Value;
use std::env;

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

fn get_config() -> Value {
    const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");
    let mut config_path = format!("{}/src/config.json", PROJECT_DIR);
    config_path = fs::read_to_string(&config_path).unwrap();
    
    serde_json::from_str(&config_path).unwrap()
}

fn main() {
    let args = Cli::parse();
    let directory = fs::read_dir(args.path);
    let config = get_config();
   
    println!("{:#}", config);

    match directory {
        Ok(_dir) => {
            get_dir_entries(_dir);
        }
        Err(e) => { println!("Error opening directory: {}", e); }
    }
}
