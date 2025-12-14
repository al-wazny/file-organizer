use clap::Parser;
use regex::Regex;
use std::ffi::{OsString};
use std::fs::{self};
use serde_json::Value;
use std::env;
use std::path::PathBuf;

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

#[derive(Debug)]
struct File {
    extension: OsString,
    file_name: OsString,
    current_path: PathBuf,
    new_path: Option<PathBuf>,
}

fn get_dir_entries(directory_path: &PathBuf) -> Option<Vec<File>> {
    fs::read_dir(directory_path).ok().map(|dir| {
        dir.filter_map(Result::ok)
            .filter(|entry| entry.file_type()
            .map(|ft| ft.is_file()).unwrap_or(false))
            .map(|entry| {
                let path = entry.path();
                File {
                    extension: path.extension().unwrap().to_owned(),
                    file_name: path.file_stem().unwrap().to_owned(),
                    current_path: path.as_path().to_owned(),
                    new_path: None,
                }
            }).collect()
    })
}


fn get_config() -> Value {
    const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");
    let mut config_path = format!("{}/src/config.json", PROJECT_DIR);
    config_path = fs::read_to_string(&config_path).unwrap();
    
    serde_json::from_str(&config_path).unwrap()
}


// search json recursively and return configured path if given target is set inside config.json
fn get_configured_path(json: &Value, file: &File, path: &String) -> Option<String> {
    match json {
        Value::Array(arr) => {
            for item in arr.iter() {
                if let Some(p) = get_configured_path(item, file, path) {
                    return Some(p);
                }
            }
            None
        }
        Value::Object(obj) => {
            for (key, value) in obj.iter() {
                let new_path = if path.is_empty() { key.to_string() } else { format!("{}/{}", path, key) };
                if let Some(p) = get_configured_path(value, file, &new_path) {
                    return Some(p);
                }
            }
            None
        }
        _ => {
            // (fix) regex needs to be specified in the config file before the file extensions
            // (chore) clean this messy shit up later on to make it somewhat human readable
            if json.as_str() == file.extension.to_str() || 
                Regex::new(json.as_str()?).unwrap().is_match(file.file_name.to_str()?) {
                Some(path.to_owned()) 
            } else { None } 
        }
    }
}


fn get_absolute_path(rel_path: &String) -> PathBuf {
    let relative = PathBuf::from(rel_path);
    let mut absolute_path = env::home_dir().unwrap();
    absolute_path.push(&relative);

    absolute_path
}


fn main() {
    let args = Cli::parse();
    let directory_path = PathBuf::from(args.path);
    
    if let Some(mut entries) = get_dir_entries(&directory_path) {
        entries.iter_mut().for_each(|entry| {
            let config = get_config();

            if let Some(new_path) = get_configured_path(&config, &entry, &"".into()) {
                let new_absolute_path = get_absolute_path(&new_path);
                entry.new_path = Some(new_absolute_path);
                println!("{:#?}", entry);
            }
        });
    } 
}
