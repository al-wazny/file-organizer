use crate::item::default::ItemCollector;
use crate::File;
use crate::Tree;
use std::ffi::OsString;
use std::io::{BufWriter, Stdout};

#[derive(Debug)]
pub struct WalkDir<'a> {
    pub tree: &'a mut Tree,
    pub entries: &'a Vec<File>,
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
        entries: &'a Vec<File>,
        std_out: &'a mut BufWriter<Stdout>,
        total: &'a mut Totals,
    ) -> WalkDir<'a> {
        WalkDir {
            tree,
            entries,
            std_out,
            total,
        }
    }

    /// Walk the whole directories
    #[inline(always)]
    pub(crate) fn walk(&mut self) {
        let depth_limit: usize = 2;

        self.entries.iter().enumerate().for_each(|(index, entry)| {
            // todo maybe remove the first if statement
            if check_hidden_file(&entry.name) {
                self.total.size += 1;
            } else if self.tree.config.depth <= depth_limit {
                Tree::print_tree(self, index, self.entries.len());

                if let Some(collector) = ItemCollector::new(entry, &self.tree.config.depth) {
                    collector.get_item(self);
                }

                self.tree.config.nodes.pop();
            }
        });
    }
}

fn check_hidden_file(check_hidden: &OsString) -> bool {
    check_hidden.to_string_lossy().starts_with('.')
}
