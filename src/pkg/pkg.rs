use crate::{ast::Ast, token::FileSet, types::TypeContext};

pub struct Package {
    pub name: String,
    pub fs: FileSet,
    pub ast: Ast,
    pub ctx: TypeContext,
}

impl Package {
    pub fn new(name: String, fs: FileSet, ast: Ast, ctx: TypeContext) -> Self {
        Self { name, fs, ast, ctx }
    }
}
