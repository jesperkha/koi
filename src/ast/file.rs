use core::fmt;
use std::{collections::HashSet, ffi::OsStr, path::PathBuf};

use crate::{
    ast::{Ast, Printer},
    module::ModulePath,
    token::{Source, Token},
};

/// A File represents a parsed source file, containing its AST, source code,
/// and other metadata about the file itself.
#[derive(Debug)]
pub struct File {
    pub filename: String,
    pub filepath: String,
    pub empty: bool,
    pub ast: Ast,
}

impl File {
    pub fn new(source: &Source, ast: Ast) -> Self {
        File {
            empty: ast.decls.is_empty() && ast.imports.is_empty(),
            filename: String::from(
                PathBuf::from(&source.filepath)
                    .file_name()
                    .unwrap_or(OsStr::new(""))
                    .to_string_lossy(),
            ),
            filepath: source.filepath.clone(),
            ast,
        }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Printer::to_string(&self.ast))
    }
}

/// An Import describes a import at the top of a file. It contains the import
/// path, the symbols imported, and the alias it should be bound to.
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Import {
    pub modpath: ModulePath,
    pub symbols: Vec<String>,
    pub alias: Option<String>,
}

/// A FileSet is a collection of Files part of the same module. The imports
/// set is a list of all imports across all source files in the set. These
/// must be type checked before this fileset can be processed further.
pub struct FileSet {
    /// Path to this fileset from root.
    pub path: String,
    pub modpath: ModulePath,
    pub imports: HashSet<Import>,
    pub files: Vec<File>,
}

impl FileSet {
    /// Create new file set from File list. List must contain at least one file.
    pub fn new(modpath: ModulePath, files: Vec<File>) -> Self {
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
                    modpath: ModulePath::new(import_path),
                    symbols: imp.imports.iter().map(Token::to_string).collect(),
                    alias: imp.alias.as_ref().map(|t| t.to_string()),
                });
            }
        }

        let filepath = files[0].filepath.clone();
        Self {
            modpath,
            path: filepath,
            imports,
            files,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}
