use crate::{ast::Ast, token::FileSet, types::TypeContext};

pub struct Package {
    /// Name of package. eg. 'main'
    pub name: String,
    /// Relative path to package from project root
    pub filepath: String,
    pub fs: FileSet,
    pub ast: Ast,
    pub ctx: TypeContext,
}

impl Package {
    pub fn new(name: String, filepath: String, fs: FileSet, ast: Ast, ctx: TypeContext) -> Self {
        Self {
            name,
            filepath,
            fs,
            ast,
            ctx,
        }
    }

    pub fn filenames(&self) -> Vec<&str> {
        self.fs
            .files
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<&str>>()
    }

    pub fn name_as(&self, path: &str, extention: &str) -> String {
        assert!(!extention.starts_with("."));
        assert!(!path.ends_with("/"));
        format!("{}/{}.{}", path, self.name, extention)
    }
}
