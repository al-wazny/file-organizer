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

fn get_dir_entries(entries: ReadDir) -> Vec<String> {
    let mut extensions = vec![];

    for entry in entries {
        if let Ok(_entry) = entry {
            let path = _entry.path();

            if let Some(extension) = path.extension() {
                println!("{:?}", extension);
                extensions.push(extension.display().to_string());
            }
        }
    }

    extensions
}

fn get_config() -> Value {
    const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");
    let mut config_path = format!("{}/src/config.json", PROJECT_DIR);
    config_path = fs::read_to_string(&config_path).unwrap();
    
    serde_json::from_str(&config_path).unwrap()
}

fn find_value(c: &Value, target: &Value) -> bool {
    match c {
        Value::Array(arr) => arr.iter().any(|item| find_value(item, target)),
        Value::Object(obj) => obj.values().any(|item| find_value(item, target)),
        _ => c == target,
    }
}

fn main() {
    let args = Cli::parse();
    let directory = fs::read_dir(args.path);
    let config = get_config();
   
    println!("{:#}", config);
    // check if a given file extension has a path configured
    println!("{}", find_value(&config, &Value::String("bar".into())));

    match directory {
        Ok(_dir) => {
            get_dir_entries(_dir);
        }
        Err(e) => { println!("Error opening directory: {}", e); }
    }
}
