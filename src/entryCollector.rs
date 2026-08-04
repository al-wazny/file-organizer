#[warn(unused_imports)]
use crate::env;
use crate::item::all;
use crate::walker;
use clap::Error;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::path;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::result;

#[derive(Debug)]
pub struct EntryCollector {
    pub json_config: Value,
    pub search_path: PathBuf,
    pub files: Option<Vec<EntryType>>,
    pub tree_result: Option<HashMap<PathBuf, Vec<PathBuf>>>, //?mayeb use type Vec<Vec<EntryType>>
}

#[derive(Debug)]
enum EntryType {
    Dir(Entry),
    File(Entry),
}

impl EntryType {
    /// Returns the path to use for depth/prefix comparisons.
    /// `None` if this is a File entry whose destination hasn't been resolved yet.
    fn resolved_path(&self) -> Option<&PathBuf> {
        match self {
            EntryType::Dir(path) => Some(&path.current_path),
            EntryType::File(entry) => entry.new_path.as_ref(),
        }
    }

    fn is_dir_variant(&self) -> bool {
        matches!(self, EntryType::Dir(_))
    }
}

#[derive(Debug, Clone)]
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

    // Ugly ass looking function right here, nigga
    pub fn get_configured_entries(mut self) -> EntryCollector {
        match self.get_dir_entries() {
            Ok(entries) => {
                let updated_entries = self.set_configured_new_path(entries);
                let entries_path_build_up = self.add_parent_dirs(&updated_entries);

                self.correct_order(&entries_path_build_up);
                println!("{:#?}", &entries_path_build_up);

                // STOP don't move forward before completing previous todo
                // let result_tree = self.create_result_tree(&foo); // todo return Vec<Vec<PathBuf>>

                // let walker_struct = self.create_walker_struct(&updated_files);

                //println!("{:#?}", walker_struct);*/
                // self.files = Some(updated_files);
                //self.tree_result = Some(walker_struct);
            }
            Err(_) => println!("couldn't get the files from given directory"),
        }

        self
    }

    // ?maybe add a second param for the recursion (name: dirs)
    // ?maybe change param type to Vec<&PathBuf>
    // ?maybe create the dir structure of the config.json even if some dirs stay empty
    // ?maybe passing in all the paths creates a lot of overhead
    pub fn correct_order(&self, paths: &Vec<EntryType>) {
        for entry in paths.iter() {
            let Some(path) = entry.resolved_path() else {
                continue;
            };
            let dir_depth = path.components().count();
            // 3. for each dir: list only the immediat content (current_depth + 1)
            let path_content: Vec<&EntryType> = paths
                .iter()
                .filter(|p| {
                    let Some(cmp) = p.resolved_path() else {
                        return false; // unresolved File entries can't match
                    };
                    cmp.components().count() == dir_depth + 1 && cmp.starts_with(path)
                })
                .collect();

            let path_dirs: Vec<&PathBuf> = path_content
                .into_iter()
                .filter(|p| p.is_dir_variant())
                .filter_map(|p| p.resolved_path())
                .collect();

            // if !path_dirs.is_empty() {
            //     println!("dirs {:#?}", path_dirs);
            // }
        }
    }

    pub fn add_parent_dirs(&self, entries: &Vec<Entry>) -> Vec<EntryType> {
        let mut result: Vec<EntryType> = Vec::new();
        let mut seen = HashSet::new();

        for entry in entries {
            let path = entry.new_path.as_ref();
            if path.is_none() {
                continue;
            };

            let mut parents = Vec::new();
            let mut current = path.unwrap().parent();

            while let Some(parent) = current {
                if parent.as_os_str().is_empty() {
                    break;
                }

                parents.push(parent.to_path_buf());
                current = parent.parent();
            }

            // Add parents in the correct order
            for parent in parents.into_iter().rev() {
                if seen.insert(parent.clone()) {
                    result.push(EntryType::Dir(Entry {
                        current_path: parent,
                        new_path: None,
                    }));
                }
            }

            // Add the original path
            if seen.insert(path.unwrap().clone()) {
                result.push(EntryType::File(entry.clone()));
            }
        }

        result
    }

    fn create_walker_struct(&self, entries: &Vec<Entry>) -> HashMap<PathBuf, Vec<PathBuf>> {
        let mut tree: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new(); //? final vec to be returned as function call
        let mut new_paths: Vec<PathBuf> = entries
            .iter()
            .filter_map(|entry| entry.new_path.clone())
            .collect();

        // Create a HashMap to store path (index) pointing to its contents (value)
        if let Some(home_dir) = env::home_dir() {
            let mut path_parts = PathBuf::from("/");
            for c in home_dir.components() {
                if let Component::Normal(s) = c {
                    path_parts.push(s);
                    new_paths.push(path_parts.clone());
                }
            }
        }
        new_paths.sort_by_key(|path| path.components().count());
        for (index, path) in new_paths.into_iter().enumerate() {
            if let Some(parent) = path.parent() {
                let mut parent_path = parent.to_path_buf();
                let mut current_path = path.to_path_buf();

                tree.entry(parent_path)
                    .or_insert_with(Vec::new)
                    .push(current_path);
            }
        }

        tree
    }

    fn get_dir_entries(&self) -> Result<Vec<Entry>, std::io::Error> {
        let mut res: Vec<Entry> = Vec::new();
        let cwd = env::current_dir()?;

        for entry in fs::read_dir(&self.search_path)? {
            let entry = entry?;

            let abs_path = match entry.path().is_absolute() {
                true => entry.path(),
                false => env::current_dir()?.join(entry.path()),
            };

            let entry = Entry {
                current_path: abs_path,
                new_path: None,
            };
            res.push(entry);
        }

        Ok(res)
    }

    fn set_configured_new_path(&self, mut files: Vec<Entry>) -> Vec<Entry> {
        let path = String::new();
        for file in files.iter_mut() {
            file.new_path = self.get_configured_path(&self.json_config, &file, &path);
        }

        files
        // files.into_iter().map(EntryType::File).collect()
    }

    fn create_result_tree(&self, files: &Vec<PathBuf>) -> Vec<PathBuf> {
        //TODO create stack data structure (last in first out)
        todo!("sort files in correct order and group everything")
    }

    // todo make path parameter of type pathbuf
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
