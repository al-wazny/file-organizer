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

// search json recursively and return configured path if target is set inside config.json
fn get_configured_path(json: &Value, target: &Value, path: String) -> Option<String> {
    match json {
        Value::Array(arr) => {
            for (_, item) in arr.iter().enumerate() {
                let new_path = format!("{}", path);
                if let Some(p) = get_configured_path(item, target, new_path) {
                    return Some(p);
                }
            }
            None
        }
        Value::Object(obj) => {
            for (key, value) in obj.iter() {
                let new_path = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{}/{}", path, key)
                };
                if let Some(p) = get_configured_path(value, target, new_path) {
                    return Some(p);
                }
            }
            None
        }
        _ => { if json == target { Some(path) } else { None } }
    }
}

fn main() {
    let args = Cli::parse();
    let directory = fs::read_dir(args.path);
    let config = get_config();
   
    let extension = "foo";
    let searched_extension = Value::String(extension.to_string());

    println!("{:#?}", config);
    if let Some(path) = get_configured_path(&config, &searched_extension, "".into()) {
        println!("{}/", path);
    } else {
        todo!("handle files with non configured extensions")
    }

    match directory {
        Ok(_dir) => {
            get_dir_entries(_dir);
        }
        Err(e) => { println!("Error opening directory: {}", e); }
    }
}
