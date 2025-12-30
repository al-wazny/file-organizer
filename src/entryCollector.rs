#[warn(unused_imports)]
use crate::env;
use crate::walker;
use clap::Error;
use regex::Regex;
use serde_json::Value;
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
    pub tree_result: Option<Vec<(PathBuf, Vec<PathBuf>)>>, //TODO create hashmap or something to use later for print_tree
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

    pub fn get_configured_entries(mut self) -> EntryCollector {
        match self.get_dir_entries() {
            Ok(files) => {
                // maybe need a for loop?
                let updated_files = self.set_configured_new_path(files);
                let result_tree = self.create_result_tree(&updated_files); // todo renmae methode
                let walker_struct = self.create_walker_struct(&updated_files);

                //println!("{:#?}", walker_struct);
                self.files = Some(updated_files);
                self.tree_result = Some(walker_struct);

            }
            Err(_) => println!("couldn't get the files from given directory")
        }
        
        self
    }

    fn create_walker_struct(&self, entries: &Vec<Entry>) -> Vec<(PathBuf, Vec<PathBuf>)> {
        //let mut map: HashMap<String, Vec<PathBuf>> = HashMap::new(); //todo rename this variable
        let mut tree: Vec<Vec<PathBuf>> = Vec::new();
        let mut path_siblings: Vec<PathBuf> = Vec::new();
        let mut new_paths: Vec<PathBuf> = entries
                                            .iter()
                                            .filter_map(|entry| entry.new_path.clone())
                                            .collect();
                    
        // Create a HashMap to store parent -> children mapping
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
        println!("new_paths {:#?}", &new_paths);
        
        // TODO create vec based on components length and max one child
        let mut all_paths: Vec<PathBuf> = Vec::new();
        for (index, path) in new_paths.into_iter().enumerate() {
            let home_dir = PathBuf::new();
            if let Some(parent) = path.parent() {
                let mut parent_path = parent.to_path_buf();
                let mut current_path = parent.to_path_buf():
            }
            
            
            //if parent_path.components().count() == index {
                //println!("path: {:#?}; index: {:#?}", &parent_path, &index);
                //} else if current_path.components().count() == index {
                    //println!("path: {:#?}; index: {:#?}", &current_path, &index);
                    //} 
        }
        
        todo!()
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

    fn create_result_tree(&self, files: &Vec<Entry>) -> Vec<PathBuf> {
        let mut dir_fir_childss = Vec::new();
        for result in files.iter() {
            let path = result.new_path.as_ref().unwrap().as_path();
            let abs_new_path = env::current_dir().unwrap().join(path);

            dir_fir_childss.push(abs_new_path);
        }
        dir_fir_childss
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
