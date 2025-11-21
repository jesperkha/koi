use crate::{
    ast::{Decl, FileSet},
    types::TypeContext,
};

pub struct Package {
    name: String,
    ctx: TypeContext,
    nodes: Vec<Decl>,
    fs: FileSet,
}

impl Package {
    pub fn new(name: String, mut fs: FileSet, ctx: TypeContext) -> Self {
        // Join ASTs
        let nodes = fs
            .files
            .iter_mut()
            .map(|f| std::mem::take(&mut f.ast.decls))
            .flatten()
            .collect::<Vec<_>>();

        Self {
            nodes,
            name,
            ctx,
            fs,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.fs.path
    }

    pub fn context(&self) -> &TypeContext {
        &self.ctx
    }

    pub fn nodes(&self) -> &[Decl] {
        &self.nodes
    }

    pub fn name_as(&self, path: &str, extention: &str) -> String {
        assert!(!extention.starts_with("."));
        assert!(!path.ends_with("/"));
        format!("{}/{}.{}", path, self.name, extention)
    }
}
