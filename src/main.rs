#![allow(dead_code, unused_variables)]
use crate::entryCollector::EntryCollector;
use crate::tree::{Branch, Config, Tree};
use crate::walker::{Totals, WalkDir};
use clap::Parser;
use serde_json::Value;
use std::env;
use std::fs::{self};
use std::io;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

mod entryCollector;
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

fn get_config() -> Value {
    const PROJECT_DIR: &str = env!("CARGO_MANIFEST_DIR");
    let mut config_path = format!("{}/src/config.json", PROJECT_DIR);
    config_path = fs::read_to_string(&config_path).unwrap();

    // todo add match statement to check for syntax errors or anything
    serde_json::from_str(&config_path).unwrap()
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
    WalkDir::new(&mut tree, Path::new("."), &mut std_out, &mut totals).walk();
}

fn main() {
    let args = Cli::parse();
    let directory_path = PathBuf::from(args.path);
    let config = get_config();
    let collector = EntryCollector::new(config, directory_path);
    let _ = collector.get_configured_entries();

    run_tree();

    // todo maybe use a match statement for better redablity
    // if let Some(mut entries) = get_dir_entries(&directory_path) {
    //     let config = get_config();
    //     for entry in entries.iter_mut() {
    //         if let Some(new_path) = get_configured_path(&config, entry, &"".into()) {
    //             entry.new_path = Some(new_path);
    //         }
    //     }
    //     println!("{:#?}", entries);
    //     // let foo = vec![
    //     //     File {}
    //     // ];
    //     // todo parameter should look like ->
    //     // 0: [0: Dokumente];
    //     // 1: [0: Dokumente/Anschreiben];
    //     // 2: [0: Dokumente/Anschreiben/file.txt; 1: Dokumente/Anschreiben/file2.txt];
    //     run_tree();
    // } else {
    //     println!("given path either doesn't exist or doesn't contain any files or directories")
    // }
}
