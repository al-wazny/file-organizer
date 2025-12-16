use crate::File;
use crate::WalkDir;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

pub struct ItemCollector {
    pub name: OsString,
    pub path: PathBuf,
    pub depth: usize,
    pub file_type: OsString,
    pub size: u64,
}

impl ItemCollector {
    #[inline(always)]
    pub fn new(entry: &File, depth: &usize) -> Option<ItemCollector> {
        // todo maybe stop returning Option<>
        if entry.new_path.is_some() {
            Some(ItemCollector {
                name: entry.name.clone(),
                path: entry.new_path.clone().unwrap(),
                depth: depth.to_owned(),
                file_type: entry.extension.clone(),
                size: 0,
            })
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_item(&self, walk: &mut WalkDir<'_>) {
        if self.path.is_dir() {
            self.process_dir(walk);
        } else {
            self.process_file(walk);
        }

        walk.total.size += self.size;
    }

    // TODO: 'process_dir' and 'process_file' should be a trait?
    #[inline(always)]
    fn process_dir(&self, walk: &mut WalkDir<'_>) {
        write!(walk.std_out, " ──> {}", &self.path.display()).unwrap();

        // Create newline
        writeln!(walk.std_out).unwrap();

        walk.total.directories += 1;

        // Add 1 as we want to traverse the next folder depth
        walk.tree.config.depth += 1;

        // -----------------------------
        // (INFO) this is a recursive traversale to display the tree structure
        // WalkDir::new(walk.tree, &self.entries, walk.std_out, walk.total1).walk();
        // -----------------------------

        // Subtract 1 as we fall back from DFS
        // Without this, the depth for current folder is not accurate
        walk.tree.config.depth -= 1;
    }

    #[inline(always)]
    fn process_file(&self, walk: &mut WalkDir<'_>) {
        write!(walk.std_out, "{}", &self.name.to_string_lossy()).unwrap_or_default();

        write!(walk.std_out, " ──> {}", &self.path.display()).unwrap();
        println!("{}", self.path.display());

        // Create newline
        writeln!(walk.std_out).unwrap();

        walk.total.files += 1;
    }
}
