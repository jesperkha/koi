use std::{collections::HashMap, hash::Hash};

use crate::{module::Exports, types::TypedAst};

pub enum ModuleKind {
    Stdlib,
    User,
    ThirdParty,
}

pub type ModuleId = usize;

pub fn invalid_mod_id() -> ModuleId {
    return usize::MAX;
}

/// Module path wraps a string module path (app.foo.bar) and provides methods
/// to get the path itself or the module name (the last name in the path).
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ModulePath(String);

impl ModulePath {
    pub fn new(s: String) -> Self {
        assert!(!s.is_empty(), "cannot have empty module path");
        Self(s)
    }

    pub fn new_str(s: &str) -> Self {
        assert!(!s.is_empty(), "cannot have empty module path");
        Self(s.to_owned())
    }

    pub fn name(&self) -> &str {
        &self.0.split(".").last().unwrap() // asserted
    }

    pub fn path(&self) -> &str {
        &self.0
    }
}

pub struct CreateModule {
    pub modpath: ModulePath,
    pub filepath: String,
    pub ast: TypedAst,
    pub exports: Exports,
    pub kind: ModuleKind,
}

/// A Module is a self-contained compilation unit. It contains the combined
/// typed AST of all files in the module and all exported symbols.
pub struct Module {
    pub id: ModuleId,
    /// The id of this modules parent. 0 means this is root.
    pub parent: ModuleId,
    /// The module path, eg. app.some.mod
    /// The module name can be fetched from the module path.
    pub modpath: ModulePath,
    /// The relative path from src to this module.
    pub path: String,
    /// The fully typed AST generated from files in this module.
    pub ast: TypedAst,
    /// All symbols exported by this module.
    pub exports: Exports,
    /// What type of module this is.
    pub kind: ModuleKind,
}

impl Module {
    pub fn name(&self) -> &str {
        self.modpath.name()
    }
}

pub struct ModuleGraph {
    pub modules: Vec<Module>,
    /// Indecies in modules vec
    pub cache: HashMap<String, ModuleId>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        ModuleGraph {
            modules: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// Create a new module and add it to the graph.
    pub fn add(&mut self, m: CreateModule, parent: ModuleId) -> &Module {
        let id = self.modules.len();
        self.modules.push(Module {
            id,
            parent,
            modpath: m.modpath,
            path: m.filepath,
            ast: m.ast,
            exports: m.exports,
            kind: m.kind,
        });

        let module = &self.modules[id];
        self.cache.insert(module.modpath.path().to_owned(), id);
        module
    }

    pub fn get(&self, id: ModuleId) -> Option<&Module> {
        assert!(id != invalid_mod_id(), "invalid mod id");
        self.modules.get(id)
    }

    /// Resolve a module path to a Module, referenced from the internal array.
    pub fn resolve(&self, modpath: &ModulePath) -> Result<&Module, String> {
        self.cache
            .get(modpath.path())
            .map_or(Err(format!("could not resolve module path")), |id| {
                Ok(&self.modules[*id])
            })
    }
}
