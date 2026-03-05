use std::collections::HashMap;

use crate::{
    module::{ModulePath, NamespaceList, SymbolId, SymbolList},
    types::TypedAst,
    util::FilePath,
};

pub type ModuleId = usize;

/// A Module is a self-contained compilation unit. It contains the combined
/// typed AST of all files in the module and all exported symbols.
pub struct Module {
    pub id: ModuleId,
    /// What type of module this is.
    pub kind: ModuleKind,
    /// The module path, eg. app.some.mod
    /// The module name can be fetched from the module path.
    pub modpath: ModulePath,
    /// List of symbols declared and used within this module.
    pub symbols: SymbolList,
    /// List of modules this module depends on.
    pub deps: Vec<ModuleId>,
}

pub enum ModuleKind {
    /// Source module are created from the source code of the current project.
    /// These modules are built into the final executable/library.
    Source(SourceModule),

    /// External modules are external libraries, such as the standard library,
    /// and are skipped when building, as they are pre-compiled.
    External,
}

pub struct SourceModule {
    /// The relative path from src to this module.
    /// For package modules this is the filepath to the linkable object file.
    pub filepath: FilePath,
    /// List of files in this source module.
    pub files: Vec<ModuleSourceFile>,
}

pub struct ModuleSourceFile {
    /// The files name.
    pub filename: String,
    /// The fully typed AST generated from the File ast.
    pub ast: TypedAst,
    /// Namespaces this file uses.
    pub namespaces: NamespaceList,
}

impl Module {
    pub fn name(&self) -> &str {
        self.modpath.path() // always non-empty in usecases
    }

    pub fn is_main(&self) -> bool {
        self.modpath.path() == "main"
    }

    /// Reports whether this module should be built (produce IR/codegen) or not.
    pub fn should_be_built(&self) -> bool {
        matches!(self.kind, ModuleKind::Source(_))
    }

    /// Collect all exported symbols from this module.
    pub fn exports(&self) -> HashMap<&String, SymbolId> {
        self.symbols
            .symbols()
            .iter()
            .filter(|(_, sym)| sym.exported)
            .map(|(name, sym)| (name, sym.id))
            .collect::<_>()
    }
}
