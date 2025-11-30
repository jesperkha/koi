use core::fmt;
use std::{collections::HashSet, ffi::OsStr, path::PathBuf};

use crate::{
    ast::{Ast, Printer, Visitable, Visitor},
    token::{Source, Token},
};

/// Unique package identifier (full import name, eg. app.server.util)
#[derive(Hash, Eq, Clone, PartialEq, Debug)]
pub struct PackageID(pub String);

impl fmt::Display for PackageID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct FileMeta {
    pub filename: String,
    pub filepath: String,
}

#[derive(Debug)]
pub struct File {
    pub meta: FileMeta,
    pub package: String,
    pub ast: Ast,
    pub src: Source,
}

impl File {
    pub fn new(package: String, src: Source, ast: Ast) -> Self {
        File {
            package,
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

    /// Walks the AST and applites the visitor to each node.
    pub fn walk<R>(&self, visitor: &mut dyn Visitor<R>) {
        for node in &self.ast.decls {
            node.accept(visitor);
        }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Printer::to_string(self))
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Import {
    pub name: PackageID,
    pub symbols: Vec<String>,
    pub alias: Option<String>,
}

/// A FileSet is a collection of ASTs (Files). The imports vector is a list of
/// all imports across all source files in the set. These must be type checked
/// before this fileset can be processed further.
pub struct FileSet {
    pub path: String,
    pub package_id: PackageID,
    pub imports: HashSet<Import>,
    pub files: Vec<File>,
}

impl FileSet {
    /// Create new file set from File list. List must contain at least one file.
    pub fn new(files: Vec<File>) -> Self {
        assert!(files.len() > 0, "files list must contain at least one file");

        let mut imports = HashSet::new();

        for file in &files {
            for imp in &file.ast.imports {
                let pkg_id = PackageID(
                    imp.names
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join("."),
                );
                imports.insert(Import {
                    name: pkg_id,
                    symbols: imp.imports.iter().map(Token::to_string).collect(),
                    alias: imp.alias.as_ref().map(|t| t.to_string()),
                });
            }
        }

        let package_id = PackageID(files[0].package.clone());
        let filepath = files[0].src.filepath.clone();

        Self {
            path: filepath,
            package_id,
            imports,
            files,
        }
    }
}
