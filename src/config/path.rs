use crate::util::FilePath;

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
    root: FilePath,
}

impl PathManager {
    pub fn new(root: FilePath) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &FilePath {
        &self.root
    }

    /// Path to library directory containing koi builtin libraries.
    pub fn library_path(&self) -> FilePath {
        self.root().join("lib")
    }

    /// Path to library directory containing external libraries.
    pub fn external_library_path(&self) -> FilePath {
        self.root().join("external")
    }
}
