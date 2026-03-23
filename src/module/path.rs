use std::fmt::Display;

use crate::{ast::ImportNode, util::FilePath};

/// Module path wraps a string module path (app.foo.bar) and provides methods
/// to get the path itself or the module name (the last name in the path).
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ModulePath {
    // Examples values for the full module path lib.socket.common.util
    prefix: String,  // lib
    package: String, // socket
    path: String,    // common.util
    is_main: bool,
}

impl ModulePath {
    pub fn new(prefix: String, package: String, path: String) -> Self {
        if !prefix.is_empty() {
            assert!(
                !package.is_empty(),
                "cannot have empty package name if prefix is non-empty",
            );
        }
        if prefix.is_empty() && package.is_empty() && path.is_empty() {
            panic!("empty module path");
        }
        Self {
            prefix,
            package,
            path,
            is_main: false,
        }
    }

    pub fn to_main(self) -> Self {
        Self {
            prefix: self.prefix,
            package: self.package,
            path: self.path,
            is_main: true,
        }
    }

    /// Create new standard library module path
    pub fn to_std(self) -> ModulePath {
        ModulePath::new("std".into(), self.package, self.path)
    }

    /// Create new external library module path
    pub fn to_lib(self) -> ModulePath {
        ModulePath::new("lib".into(), self.package, self.path)
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn is_main(&self) -> bool {
        self.is_main
    }

    pub fn import_path(&self) -> ImportPath {
        if !self.prefix.is_empty() {
            ImportPath::new(
                std::iter::once(self.prefix.as_str())
                    .chain(std::iter::once(self.package.as_str()))
                    .chain(self.path.split('.'))
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("."),
            )
        } else {
            ImportPath::from(self.path.as_str())
        }
    }

    /// Check if this module path is part of the standard library.
    pub fn is_stdlib(&self) -> bool {
        self.prefix == "std"
    }

    /// Check if this module path is an external library.
    pub fn is_library(&self) -> bool {
        self.prefix == "lib"
    }

    /// Get the module path with underscore (_) separators instead of period (.)
    pub fn to_underscore(&self) -> String {
        std::iter::once(self.prefix.as_str())
            .chain(std::iter::once(self.package.as_str()))
            .chain(self.path.split('.'))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("_")
    }

    pub fn to_header_format(&self) -> String {
        if self.path.is_empty() {
            self.package.clone()
        } else {
            format!("{}.{}", self.package, self.path)
        }
    }
}

impl From<&FilePath> for ModulePath {
    // Convert header path to module path
    // /lib/external/mylib.util.koi.h -> mylib.util
    fn from(p: &FilePath) -> Self {
        let p = p.filename().expect("expected filepath");
        let s = p.to_string();
        let s = s.trim_end_matches(".koi.h");
        let mut iter = s.split(".");
        let package = iter.next().expect("bad filepath");
        let path = iter.collect::<Vec<_>>().join(".");
        ModulePath::new("".into(), package.into(), path)
    }
}

impl From<ImportPath> for ModulePath {
    // Turns import path into module path
    // lib.mylib.core.util -> lib, mylib, core.util
    // server.router -> <empty>, <empty>, server.router
    fn from(impath: ImportPath) -> Self {
        if impath.is_library() || impath.is_stdlib() {
            let mut split = impath.path().split(".");
            let prefix = split.next().unwrap();
            let package = split
                .next()
                .expect("prefix without package name not allowed");
            let path = split.collect::<Vec<_>>().join(".");
            ModulePath::new(prefix.into(), package.into(), path)
        } else {
            ModulePath::new("".into(), "".into(), impath.path)
        }
    }
}

impl Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.import_path())
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ImportPath {
    path: String,
}

impl Display for ImportPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl ImportPath {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Get only the module name (the last identifier of the path).
    pub fn name(&self) -> &str {
        self
            .path
            .split(".")
            .last()
            .expect("called name on a non-import path")
    }

    /// Check if this module path is part of the standard library.
    pub fn is_stdlib(&self) -> bool {
        self.path.starts_with("std.")
    }

    /// Check if this module path is an external library.
    pub fn is_library(&self) -> bool {
        self.path.starts_with("lib.")
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl From<&str> for ImportPath {
    fn from(s: &str) -> Self {
        Self::new(s.into())
    }
}

impl From<String> for ImportPath {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&ModulePath> for ImportPath {
    fn from(modpath: &ModulePath) -> Self {
        modpath.import_path()
    }
}

impl From<&ImportNode> for ImportPath {
    fn from(import: &ImportNode) -> Self {
        import
            .names
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(".")
            .into()
    }
}
