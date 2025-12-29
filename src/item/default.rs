use crate::WalkDir;
use std::ffi::OsString;
use std::fs;
use std::fs::DirEntry;
use std::fs::FileType;
use std::io::Write;
use std::path::PathBuf;

pub struct ItemCollector {
    pub name: String,
    pub path: PathBuf,
    pub depth: usize,
    pub file_type: Option<FileType>,
    pub size: Option<u64>,
}

// Todo refactor this
impl ItemCollector {
    #[inline(always)]
    pub fn new(entry: &PathBuf, depth: &usize) -> ItemCollector {
        let name = entry.file_name().unwrap().to_string_lossy().to_string().to_owned();
        let path = entry.as_path().to_path_buf();
        let file_type = entry.metadata().ok().map(|file| file.file_type());
        let size =  entry.metadata().ok().map(|file| file.len());

        ItemCollector {
            name,
            path,
            depth: depth.to_owned(),
            file_type,
            size,
        }
    }

    #[inline(always)]
    pub fn get_item(&self, walk: &mut WalkDir<'_>) {
        if self.path.is_dir() {
            self.process_dir(walk);
        } else {
            self.process_file(walk);
        }

        if let Some(size) = self.size {
            walk.total.size += size;
        }
    }

    // TODO: 'process_dir' and 'process_file' should be a trait?
    #[inline(always)]
    fn process_dir(&self, walk: &mut WalkDir<'_>) {
        write!(walk.std_out, "{} ──> {}", &self.name, &self.path.display()).unwrap();

        // Create newline
        writeln!(walk.std_out).unwrap();

        walk.total.directories += 1;

        // Add 1 as we want to traverse the next folder depth
        walk.tree.config.depth += 1;

        // -----------------------------
        // (INFO) this is a recursive traversale to display the tree structure
        WalkDir::new(walk.tree, &self.path, walk.std_out, walk.total, walk.result_tree).walk();
        // -----------------------------

        // Subtract 1 as we fall back from DFS
        // Without this, the depth for current folder is not accurate
        walk.tree.config.depth -= 1;
    }

    #[inline(always)]
    fn process_file(&self, walk: &mut WalkDir<'_>) {
        write!(walk.std_out, "{}", &self.name.to_string()).unwrap_or_default();

        write!(walk.std_out, " ──> {}", &self.path.display()).unwrap();

        // Create newline
        writeln!(walk.std_out).unwrap();

        walk.total.files += 1;
    }
}
