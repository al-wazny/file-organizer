#![allow(dead_code, unused_variables)]
use crate::tree::{Branch, Config, Tree};
use crate::walker::{Totals, WalkDir};
use clap::Parser;
use regex::Regex;
use serde_json::Value;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self};
use std::io;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

mod item;
mod tree;
mod walker;

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
pub struct File {
    extension: OsString,
    name: OsString,
    current_path: PathBuf,
    new_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Entry {
    name: String,
    is_file: bool,
    file: Option<File>,
}

fn run_tree() {
    let config = Config::new(Vec::with_capacity(5_000), 1);
    let mut std_out = BufWriter::new(io::stdout());
    let mut tree = Tree::new(config, Branch::new());
    // todo implement the default trait
    let mut totals = Totals {
        directories: 0,
        files: 0,
        size: 0,
    };
    // Iterate branches
    // (Info) the flag is needed to check if the depth limit is reached
    // it traverses the each directory till it reaches a branch, but you're already giving him
    // the entire path which won't display the entire tree structur
    WalkDir::new(
        &mut tree,
        Path::new("/home/lalwazny"),
        &mut std_out,
        &mut totals,
    )
    .walk();
}

fn get_dir_entries(directory_path: &PathBuf) -> Option<Vec<File>> {
    fs::read_dir(directory_path).ok().map(|dir| {
        dir.filter_map(Result::ok)
            // .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            // .filter(|entry| entry.path().extension().is_some())
            .map(|entry| {
                // if let path = entry.path() {
                //
                // }
                let path = entry.path();
                let extension = path.extension().unwrap_or(OsStr::new(""));
                let file_name = path.file_name().unwrap();
                let current_path = path.as_path();

                File {
                    extension: extension.to_owned(),
                    name: file_name.to_os_string(),
                    current_path: current_path.to_owned(),
                    new_path: None,
                }
            })
            .collect()
    })
}

fn get_config() -> Value {
    const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");
    let mut config_path = format!("{}/src/config.json", PROJECT_DIR);
    config_path = fs::read_to_string(&config_path).unwrap();

    // todo add match statement to check for syntax errors or anything
    serde_json::from_str(&config_path).unwrap()
}

// search json recursively and return configured path if given target is set inside config.json
fn get_configured_path(json: &Value, file: &File, path: &String) -> Option<PathBuf> {
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
                let new_path = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{}/{}", path, key)
                };
                if let Some(p) = get_configured_path(value, file, &new_path) {
                    return Some(p);
                }
            }
            None
        }
        Value::String(value) => {
            // TODO:use json key to specifi extension and regex values and check for regex first
            let value = json.as_str().unwrap();
            let file_name = file.name.to_str()?;
            let regex = Regex::new(value).unwrap();
            let extension = file.extension.to_str().unwrap().to_uppercase();

            if value.to_uppercase() == extension || regex.is_match(file_name) {
                let rel_path = format!("{}/{}", path, file_name);
                let absolute_path = get_absolute_path(rel_path).unwrap();
                let new_path = PathBuf::from(absolute_path);
                Some(new_path)
            } else {
                None
            }
        }
        _ => None, // TODO return a default path when there's no config
    }
}

fn get_absolute_path(rel_path: String) -> Option<String> {
    let relative = PathBuf::from(rel_path);
    let mut absolute_path = env::home_dir().unwrap();
    absolute_path.push(&relative);

    Some(absolute_path.display().to_string())
}

fn main() {
    let args = Cli::parse();
    let directory_path = PathBuf::from(args.path);

    // todo maybe use a match statement for better redablity
    if let Some(mut entries) = get_dir_entries(&directory_path) {
        // println!("{:#?}", entries);
        let config = get_config();
        for entry in entries.iter_mut() {
            if let Some(new_path) = get_configured_path(&config, entry, &"".into()) {
                entry.new_path = Some(new_path);
            }
        }
        // let foo = vec![
        //     File {}
        // ];
        // todo parameter should look like ->
        // 0: [0: Dokumente];
        // 1: [0: Dokumente/Anschreiben];
        // 2: [0: Dokumente/Anschreiben/file.txt; 1: Dokumente/Anschreiben/file2.txt];
        run_tree();
    } else {
        println!("given path either doesn't exist or doesn't contain any files or directories")
    }
}
