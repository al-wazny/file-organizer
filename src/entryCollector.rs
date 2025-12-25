use crate::env;
use regex::Regex;
use serde_json::Value;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct EntryCollector {
    name: String,
    is_file: bool,
    file: Option<Entry>,
    tree_result: Vec<String>, //TODO create hashmap or something to use later for print_tree
}

#[derive(Debug)]
pub struct Entry {
    extension: OsString,
    name: OsString,
    current_path: PathBuf,
    new_path: Option<PathBuf>,
}

impl EntryCollector {
    pub fn get_configured_entries(path: &PathBuf) -> Option<Vec<EntryCollector>> {
        let entries = get_dir_entries(&path);
    }

    fn get_dir_entries(directory_path: &PathBuf) -> Option<Vec<Entry>> {
        // todo use match statement
        fs::read_dir(directory_path).ok().map(|dir| {
            dir.filter_map(Result::ok)
                // .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                // .filter(|entry| entry.path().extension().is_some())
                .map(|entry| {
                    let path = entry.path();
                    let extension = path.extension().unwrap_or(OsStr::new(""));
                    let file_name = path.file_name().unwrap();
                    let current_path = path.as_path();

                    Entry {
                        extension: extension.to_owned(),
                        name: file_name.to_os_string(),
                        current_path: current_path.to_owned(),
                        new_path: None,
                    }
                })
                .collect()
        })
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
}
