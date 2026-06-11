use core::fmt;
use std::collections::HashSet;

use koi_common::source::Source;
use koi_common::util::FilePath;

use crate::{
    nodes::{Ast, ImportNode},
    path::{ImportPath, ModulePath},
    print::Printer,
    token::Token,
};

#[derive(Debug)]
pub struct File {
    pub filename: String,
    pub filepath: FilePath,
    pub empty: bool,
    pub ast: Ast,
}

impl File {
    pub fn new(source: &Source, ast: Ast) -> Self {
        File {
            empty: ast.decls.is_empty() && ast.imports.is_empty(),
            filename: source.filepath.filename().unwrap_or("".into()),
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

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Import {
    pub impath: ImportPath,
    pub symbols: Vec<String>,
    pub alias: Option<String>,
}

pub struct FileSet {
    pub filepath: FilePath,
    pub modpath: ModulePath,
    pub imports: HashSet<Import>,
    pub files: Vec<File>,
}

impl FileSet {
    pub fn new(modpath: ModulePath, files: Vec<File>) -> Self {
        assert!(
            !files.is_empty(),
            "files list must contain at least one file"
        );

        let mut imports = HashSet::new();

        for file in &files {
            for imp in &file.ast.imports {
                imports.insert(Import {
                    impath: imp.into(),
                    symbols: imp.imports.iter().map(Token::to_string).collect(),
                    alias: imp.alias.as_ref().map(|t| t.to_string()),
                });
            }
        }

        let filepath = files[0].filepath.clone();
        Self {
            modpath,
            filepath,
            imports,
            files,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}
