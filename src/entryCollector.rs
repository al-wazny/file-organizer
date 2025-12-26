use crate::env;
use clap::Error;
use regex::Regex;
use serde_json::Value;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct EntryCollector {
    json_config: Value,
    search_path: PathBuf,
    files: Option<Vec<Entry>>,
    tree_result: Option<Vec<String>>, //TODO create hashmap or something to use later for print_tree
}

#[derive(Debug)]
pub struct Entry {
    current_path: PathBuf, // getter for filename and extension
    new_path: Option<PathBuf>,
}

impl EntryCollector {
    pub fn new(config: Value, path: PathBuf) -> EntryCollector {
        EntryCollector {
            json_config: config,
            search_path: path,
            files: None,
            tree_result: None,
        }
    }

    pub fn get_configured_entries(mut self) -> Result<(), Error> {
        if let Some(files) = self.get_dir_entries() {
            let new_files = self.set_configured_new_path(files);
            self.files = Some(new_files);
        }

        println!("{:#?}", &self.files);
        Ok(())
    }

    fn create_result_tree(&self) -> Vec<String> {
        todo!()
    }

    fn set_configured_new_path(&self, files: Vec<Entry>) -> Vec<Entry> {
        let files: Vec<Entry> = files
            .into_iter()
            .map(|mut file| {
                if let Some(new_path) =
                    self.get_configured_path(&self.json_config, &file, &String::new())
                {
                    file.new_path = Some(new_path);
                }
                file
            })
            .collect();

        files
    }

    fn get_dir_entries(&self) -> Option<Vec<Entry>> {
        let dir = fs::read_dir(&self.search_path).ok()?; // returns None if read_dir fails

        Some(
            dir.filter_map(Result::ok)
                .map(|entry| {
                    let current_path = entry.path();
                    Entry {
                        current_path,
                        new_path: None,
                    }
                })
                .collect(),
        )
    }

    // search json recursively and return configured path if given target is set inside config.json
    fn get_configured_path(&self, config: &Value, file: &Entry, path: &String) -> Option<PathBuf> {
        match config {
            Value::Array(arr) => {
                for item in arr.iter() {
                    if let Some(p) = self.get_configured_path(item, file, path) {
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
                    if let Some(p) = self.get_configured_path(value, file, &new_path) {
                        return Some(p);
                    }
                }
                None
            }
            Value::String(value) => {
                // TODO:use json key to specifi extension and regex values and check for regex first
                let value = config.as_str().unwrap();
                let file_name: &str = file.current_path.file_name().unwrap().to_str().unwrap();
                let regex = Regex::new(value).unwrap();
                let extension: String = file
                    .current_path
                    .extension()
                    .unwrap_or(OsStr::new(""))
                    .to_string_lossy()
                    .to_uppercase();

                if value.to_uppercase() == extension || regex.is_match(file_name) {
                    let rel_path = format!("{}/{}", path, file_name);
                    let absolute_path = self.get_absolute_path(rel_path).unwrap();
                    let new_path = PathBuf::from(absolute_path);
                    Some(new_path)
                } else {
                    None
                }
            }
            _ => None, // TODO return a default path when there's no config
        }
    }

    fn get_absolute_path(&self, rel_path: String) -> Option<String> {
        let relative = PathBuf::from(rel_path);
        let mut absolute_path = env::home_dir().unwrap();
        absolute_path.push(&relative);

        Some(absolute_path.display().to_string())
    }
}
