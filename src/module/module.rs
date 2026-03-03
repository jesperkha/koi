use std::collections::HashMap;

use crate::{
    module::{ModulePath, NamespaceList, Symbol, SymbolList, SymbolOrigin},
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
    // TODO: make separate list of symbols imported by name
    // to not make .exports() dependent on anything
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
    /// The fully typed AST generated from files in this module.
    pub ast: TypedAst,
    /// List of namespaces imported into this module.
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
    pub fn exports(&self) -> HashMap<&String, &Symbol> {
        self.symbols
            .symbols()
            .iter()
            .filter(|s| {
                // Is it exported?
                if !s.1.is_exported {
                    return false;
                }

                match &s.1.origin {
                    SymbolOrigin::Module(modpath) => &self.modpath == modpath,
                    SymbolOrigin::Extern(modpath) => &self.modpath == modpath,
                }
            })
            .collect::<_>()
    }
}
