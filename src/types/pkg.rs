use crate::{
    ast::{Decl, File},
    types::TypeContext,
};

pub struct Package {
    /// Name of package. eg. 'main'
    pub name: String,
    /// Relative path to package from project root
    pub filepath: String,
    pub ctx: TypeContext,
    pub files: Vec<File>,
    pub nodes: Vec<Decl>,
}

impl Package {
    pub fn new(name: String, filepath: String, mut files: Vec<File>, ctx: TypeContext) -> Self {
        // Join ASTs
        let nodes = files
            .iter_mut()
            .map(|f| std::mem::take(&mut f.ast.decls))
            .flatten()
            .collect::<Vec<_>>();

        Self {
            nodes,
            name,
            filepath,
            ctx,
            files,
        }
    }

    pub fn name_as(&self, path: &str, extention: &str) -> String {
        assert!(!extention.starts_with("."));
        assert!(!path.ends_with("/"));
        format!("{}/{}.{}", path, self.name, extention)
    }
}
