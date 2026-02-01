use std::path::PathBuf;

/// PathManager manages paths for Koi installation. Everything is relative to
/// the koi executable, which is assumed to be in the root directory.
///
/// Koi installation layout:
///
/// ```txt
/// :root:/
///     lib/       # Compiled shared libraries
///     include/   # Module header files
///     koi        # Koi executable
/// ```
pub struct PathManager {
    root: PathBuf,
}

impl PathManager {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Path to library directory containing compiled shared libraries.
    pub fn library_path(&self) -> PathBuf {
        self.root().join("lib")
    }

    /// Path to include directory containing module header files.
    pub fn include_path(&self) -> PathBuf {
        self.root().join("include")
    }
}
