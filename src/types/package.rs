use crate::types::{Decl, Exports, TypeContext, TypedAst};

pub struct Package {
    name: String,
    tree: TypedAst,
    exports: Exports,
}

impl Package {
    pub fn new(name: String, tree: TypedAst, exports: Exports) -> Self {
        Self {
            name,
            tree,
            exports,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    // pub fn path(&self) -> &str {
    //     &self.fs.path
    // }

    pub fn context(&self) -> &TypeContext {
        &self.tree.ctx
    }

    pub fn nodes(&self) -> &[Decl] {
        &self.tree.decls
    }

    pub fn name_as(&self, path: &str, extention: &str) -> String {
        assert!(!extention.starts_with("."));
        assert!(!path.ends_with("/"));
        format!("{}/{}.{}", path, self.name, extention)
    }
}
