use crate::{ast::File, types::TypeContext};

pub struct Package {
    /// Name of package. eg. 'main'
    pub name: String,
    /// Relative path to package from project root
    pub filepath: String,
    pub ctx: TypeContext,
    pub files: Vec<File>,
}

impl Package {
    pub fn new(name: String, filepath: String, files: Vec<File>, ctx: TypeContext) -> Self {
        Self {
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
