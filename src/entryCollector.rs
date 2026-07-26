#[warn(unused_imports)]
use crate::env;
use crate::item::all;
use crate::walker;
use clap::Error;
use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::path;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::result;
use std::collections::HashMap;


#[derive(Debug)]
pub struct EntryCollector {
    pub json_config: Value,
    pub search_path: PathBuf,
    pub files: Option<Vec<Entry>>,
    pub tree_result: Option<HashMap<PathBuf, Vec<PathBuf>>>//TODO create hashmap or something to use later for print_tree
}

enum EntryType {
    Dir(PathBuf),
    File(PathBuf, PathBuf), // (parent_dir, file_name)
}

#[derive(Debug)]
pub struct Entry {
    current_path: PathBuf, // getter for filename and extension
    new_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct DebugEntry {
    path: PathBuf,
    parent: PathBuf,
    index: u16,
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

    pub fn get_configured_entries(mut self) -> EntryCollector {
        match self.get_dir_entries() {
            Ok(files) => {
                let updated_files = self.set_configured_new_path(files);
                let paths: Vec<PathBuf> = updated_files
                    .iter()
                    .filter_map(|entry| entry.new_path.as_ref().cloned())
                    .collect();

                //TODO filter Entry to get Vec<PathBuf> (new_path) and build up every depth level of the dir_structure
                let foo = EntryCollector::add_parent_dirs(&paths); // todo rename add_missing_depth?
                //println!("foo {:#?}", foo);
                
                // STOP don't move forward before completing previous todo
                let result_tree = self.create_result_tree(&foo); // todo return Vec<Vec<PathBuf>>
                println!("tree {:#?}", result_tree
            
            );
                
                let walker_struct = self.create_walker_struct(&updated_files);
 
                //println!("{:#?}", walker_struct);*/
                self.files = Some(updated_files);
                //self.tree_result = Some(walker_struct);

            }
            Err(_) => println!("couldn't get the files from given directory")
        }
        
        self
    }

    pub fn add_parent_dirs(paths: &Vec<PathBuf>) -> Vec<PathBuf> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for path in paths {
            // Collect parent directories from root downwards
            let mut parents = Vec::new();

            let mut current = path.parent();
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
                    result.push(parent);
                }
            }

            // Add the original path
            if seen.insert(path.clone()) {
                result.push(path.clone());
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
        println!("{:#?}", &new_paths);
        for (index, path) in new_paths.into_iter().enumerate() {
            
            if let Some(parent) = path.parent() {


                let mut parent_path = parent.to_path_buf();
                let mut current_path = path.to_path_buf();
                
                /* println!("{:#?}", DebugEntry {
                    parent: parent_path.clone(),
                    path: current_path.clone(),
                    index: index.clone() as u16,
                }); */
                            
                tree.entry(parent_path)
                    .or_insert_with(Vec::new)
                    .push(current_path);

            }
            
        }
        
        tree
    }   

    // todo
    fn get_dir_entries(&self) -> Result<Vec<Entry>, std::io::Error> {
        let mut res: Vec<Entry> = Vec::new();
        let cwd = env::current_dir()?;

        for entry in fs::read_dir(&self.search_path)? {
            let entry = entry?;
            
            let abs_path = match entry.path().is_absolute() {
                true => entry.path(),
                false => env::current_dir()?.join(entry.path()),
            };

            let entry = Entry { current_path: abs_path, new_path: None};
            res.push(entry);
        };
        
        Ok(res)
    }

    fn set_configured_new_path(&self, mut files: Vec<Entry>) -> Vec<Entry> {
        let path = String::new();
        for file in files.iter_mut() {
            file.new_path = self.get_configured_path(&self.json_config, &file, &path);
        }
        files.sort_by_key(|entry| entry.new_path.as_ref().unwrap().components().count());
        //println!("files{:#?}", files);
        files
    }

    fn create_result_tree(&self, files: &Vec<PathBuf>) -> Vec<PathBuf> {
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
