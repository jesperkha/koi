use core::fmt;
use std::{collections::HashSet, ffi::OsStr, path::PathBuf};

use crate::{
    ast::{Ast, Printer},
    token::{Source, Token},
};

#[derive(Debug)]
pub struct FileMeta {
    pub filename: String,
    pub filepath: String,
}

/// A File represents a parsed source file, containing its AST, source code,
/// declared package name, and other metadata about the file itself.
#[derive(Debug)]
pub struct File {
    /// The declared package name in the file.
    pub package_name: String,
    pub meta: FileMeta,
    pub ast: Ast,
    pub src: Source,
}

impl File {
    pub fn new(package_name: String, src: Source, ast: Ast) -> Self {
        File {
            package_name,
            meta: FileMeta {
                filename: String::from(
                    PathBuf::from(&src.filepath)
                        .file_name()
                        .unwrap_or(OsStr::new(""))
                        .to_string_lossy(),
                ),
                filepath: src.filepath.clone(),
            },
            ast,
            src,
        }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Printer::to_string(self))
    }
}

/// An Import describes a import at the top of a file. It contains the import
/// path, the symbols imported, and the alias it should be bound to.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Import {
    pub import_path: String,
    pub symbols: Vec<String>,
    pub alias: Option<String>,
}

/// A FileSet is a collection of Files part of the same package. The imports
/// set is a list of all imports across all source files in the set. These
/// must be type checked before this fileset can be processed further.
pub struct FileSet {
    /// Path to this fileset from root.
    pub path: String,
    /// Full depenency name. Name of each parent directory from root joined by
    /// a period, e.g. "app.storage.db".
    pub import_path: String,
    /// Declared package name, e.g. 'util'.
    pub package_name: String,
    pub imports: HashSet<Import>,
    pub files: Vec<File>,
}

impl FileSet {
    /// Create new file set from File list. List must contain at least one file.
    pub fn new(depname: String, files: Vec<File>) -> Self {
        assert!(files.len() > 0, "files list must contain at least one file");

        let mut imports = HashSet::new();

        for file in &files {
            for imp in &file.ast.imports {
                let import_path = imp
                    .names
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(".");

                imports.insert(Import {
                    import_path,
                    symbols: imp.imports.iter().map(Token::to_string).collect(),
                    alias: imp.alias.as_ref().map(|t| t.to_string()),
                });
            }
        }

        let package_name = files[0].package_name.clone();
        let filepath = files[0].src.filepath.clone();

        Self {
            path: filepath,
            import_path: depname,
            package_name,
            imports,
            files,
        }
    }
}
