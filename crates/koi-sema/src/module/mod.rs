mod namespace;
mod symbols;

pub use koi_ast::{ImportPath, ModulePath};
pub use namespace::*;
pub use symbols::*;

use std::collections::HashMap;

use koi_common::util::FilePath;

use crate::types::TypedAst;

pub type ModuleId = usize;

pub struct Module {
    pub id: ModuleId,
    pub kind: ModuleKind,
    pub modpath: ModulePath,
    pub symbols: SymbolList,
}

pub enum ModuleKind {
    Source {
        filepath: FilePath,
        files: Vec<ModuleSourceFile>,
    },
    External,
}

pub struct ModuleSourceFile {
    pub filename: String,
    pub ast: TypedAst,
    pub namespaces: NamespaceList,
}

impl Module {
    pub fn name(&self) -> &str {
        self.modpath.path()
    }

    pub fn is_main(&self) -> bool {
        self.modpath.path() == "main"
    }

    pub fn should_be_built(&self) -> bool {
        matches!(self.kind, ModuleKind::Source { .. })
    }

    pub fn exports(&self) -> HashMap<&String, SymbolId> {
        self.symbols
            .symbols()
            .iter()
            .filter(|(_, sym)| sym.exported)
            .map(|(name, sym)| (name, sym.id))
            .collect::<_>()
    }

    pub fn imports(&self) -> Vec<SymbolId> {
        self.symbols
            .symbols()
            .iter()
            .filter(|(_, sym)| matches!(sym.kind, ModuleSymbolKind::Imported))
            .map(|(_, sym)| sym.id)
            .collect::<_>()
    }
}
