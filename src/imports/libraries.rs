use std::{
    collections::{HashMap, HashSet},
    fs::{self, DirEntry, ReadDir},
    io::Error,
    path::PathBuf,
};

use tracing::info;

use crate::module::ModulePath;

struct Header {
    header_path: PathBuf,
    archive_idx: usize,
}

pub struct LibrarySet {
    archives: Vec<PathBuf>,
    libs: HashMap<ModulePath, Header>,
}

impl LibrarySet {
    pub fn new() -> Self {
        Self {
            archives: Vec::new(),
            libs: HashMap::new(),
        }
    }

    /// Read a given directory and collect all libraries.
    /// Header files are mapped to their corresponding archive file.
    pub fn read_dir(&mut self, dir: &PathBuf) -> Result<(), String> {
        let libs = find_libraries(dir)?;

        for lib in libs {
            let archive_id = self.archives.len();
            self.archives.push(lib.archive);

            for header_path in lib.headers {
                let modpath = ModulePath::from(&header_path).to_lib();
                let header = Header {
                    header_path,
                    archive_idx: archive_id,
                };

                info!("Using header: {} at {:?}", modpath, header.header_path);
                self.libs.insert(modpath, header);
            }
        }

        Ok(())
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

struct Library {
    headers: Vec<PathBuf>,
    archive: PathBuf,
}

fn read_dir(dir: &PathBuf) -> Result<ReadDir, String> {
    fs::read_dir(dir).map_err(|_| format!("error: failed to read directory {:?}", dir))
}

fn get_entry(res: Result<DirEntry, Error>) -> Result<DirEntry, String> {
    res.map_err(|e| format!("error: failed to read entry: {}", e))
}

fn find_libraries(source_dir: &PathBuf) -> Result<Vec<Library>, String> {
    let mut libraries = Vec::new();

    let entries = read_dir(source_dir)?;
    for entry in entries {
        let entry = get_entry(entry)?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let mut headers: Vec<PathBuf> = Vec::new();
        let mut archive: Option<PathBuf> = None;

        let sub_entries = read_dir(&path)?;
        for file in sub_entries {
            let file = get_entry(file)?;
            let file_path = file.path();

            match file_path.extension().and_then(|e| e.to_str()) {
                Some("h") => headers.push(file_path),
                Some("a") => archive = Some(file_path),
                _ => {}
            }
        }

        if let Some(archive) = archive {
            libraries.push(Library { headers, archive });
        }
    }

    Ok(libraries)
}
