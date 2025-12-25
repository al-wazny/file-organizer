use crate::item::default::ItemCollector;
use crate::Tree;
use std::ffi::OsString;
use std::fs;
use std::fs::DirEntry;
use std::io::Write;
use std::io::{BufWriter, Stdout};
use std::path::Path;

#[derive(Debug)]
pub struct WalkDir<'a> {
    pub tree: &'a mut Tree,
    pub path: &'a Path,
    pub std_out: &'a mut BufWriter<Stdout>,
    pub total: &'a mut Totals,
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
    ) -> WalkDir<'a> {
        WalkDir {
            tree,
            path,
            std_out,
            total,
        }
    }

    /// Walk the whole directories
    #[inline(always)]
    pub(crate) fn walk(&mut self) {
        let depth_limit: usize = 2;
        let entries: Vec<_> = fs::read_dir(self.path).unwrap().collect();

        // println!("{:#?}", entries);
        entries.iter().enumerate().for_each(|(index, entry)| {
            match entry.as_ref() {
                Ok(entry) => {
                    // By default, we skip hidden_file
                    if check_hidden_file(entry) {
                        self.total.size += 1;
                    } else if self.tree.config.depth <= depth_limit {
                        Tree::print_tree(self, index, entries.len());

                        ItemCollector::new(entry, &self.tree.config.depth).get_item(self);

                        self.tree.config.nodes.pop();
                    }
                }
                Err(err) => {
                    writeln!(self.std_out, "{}", err).unwrap();
                }
            }
        });
    }
}

fn check_hidden_file(check_hidden: &fs::DirEntry) -> bool {
    check_hidden.file_name().to_string_lossy().starts_with('.')
}
