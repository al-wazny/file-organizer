#[warn(unused)]
use crate::item::default::ItemCollector;
use crate::Tree;
use std::ffi::OsString;
use std::fs;
use std::fs::DirEntry;
use std::io::Write;
use std::io::{BufWriter, Stdout};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct WalkDir<'a> {
    pub tree: &'a mut Tree,
    pub path: &'a Path,
    pub std_out: &'a mut BufWriter<Stdout>,
    pub total: &'a mut Totals,
    pub result_tree: &'a Vec<String>,
}

#[derive(Debug)]
pub struct Totals {
    pub directories: usize,
    pub files: usize,
    pub size: u64,
}

impl<'a> WalkDir<'a> {
    #[inline(always)]
    pub(crate) fn new(
        tree: &'a mut Tree,
        path: &'a Path,
        std_out: &'a mut BufWriter<Stdout>,
        total: &'a mut Totals,
        result_tree: &'a Vec<String>
    ) -> WalkDir<'a> {
        WalkDir {
            tree,
            path,
            std_out,
            total,
            result_tree
        }
    }

    /// Walk the whole directories
    #[inline(always)]
    pub(crate) fn walk(&mut self) {
        let depth_limit: usize = 2;
        //let entries: Vec<_> = fs::read_dir(self.path).unwrap().collect();

        // println!("{:#?}", entries);
        self.result_tree.iter().enumerate().for_each(|(index, entry)| {
     
            if self.tree.config.depth <= depth_limit {
                // todo calculate entries length
                Tree::print_tree(self, index, self.result_tree.len());

                ItemCollector::new(&PathBuf::from(entry), &self.tree.config.depth).get_item(self);

                self.tree.config.nodes.pop();
            }
        
    
        });
    }
}

fn check_hidden_file(check_hidden: &fs::DirEntry) -> bool {
    check_hidden.file_name().to_string_lossy().starts_with('.')
}
