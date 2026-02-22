use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::module::ModulePath;

struct Library {
    header_path: PathBuf,
    archive_idx: usize,
}

pub struct LibrarySet {
    archives: Vec<PathBuf>,
    libs: HashMap<ModulePath, Library>,
}

impl LibrarySet {
    pub fn new() -> Self {
        Self {
            archives: Vec::new(),
            libs: HashMap::new(),
        }
    }

    pub fn get_header_path(&self, modpath: &ModulePath) -> Option<&PathBuf> {
        self.libs.get(modpath).map(|lib| &lib.header_path)
    }

    pub fn get_archive_path(&self, modpath: &ModulePath) -> Option<&PathBuf> {
        self.libs
            .get(modpath)
            .map(|lib| &self.archives[lib.archive_idx])
    }

    pub fn modpaths(&self) -> HashSet<&ModulePath> {
        self.libs.keys().collect()
    }
}
