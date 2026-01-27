use std::{collections::HashMap, hash::Hash};

use crate::{
    module::{NamespaceList, Symbol, SymbolList, SymbolOrigin},
    types::TypedAst,
};

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
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
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

    /// Get only the module name (the last identifier of the path).
    pub fn name(&self) -> &str {
        &self.0.split(".").last().unwrap() // asserted
    }

    /// Get the full module path.
    pub fn path(&self) -> &str {
        &self.0
    }

    /// Get the module path with underscore (_) separators instead of period (.)
    pub fn path_underscore(&self) -> String {
        String::from(&self.0).replace(".", "_")
    }
}

pub struct CreateModule {
    pub modpath: ModulePath,
    pub filepath: String,
    pub ast: TypedAst,
    pub kind: ModuleKind,
    pub symbols: SymbolList,
    pub namespaces: NamespaceList,
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
    /// What type of module this is.
    pub kind: ModuleKind,
    /// List of symbols declared and used within this module.
    pub symbols: SymbolList,
    /// List of namespaces imported into this module.
    pub namespaces: NamespaceList,
}

impl Module {
    pub fn name(&self) -> &str {
        self.modpath.name()
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

pub struct ModuleGraph {
    pub modules: Vec<Module>,
    /// Indecies in modules vec
    pub cache: HashMap<String, ModuleId>,

    /// id of main module
    main_id: ModuleId,
}

impl ModuleGraph {
    pub fn new() -> Self {
        ModuleGraph {
            modules: Vec::new(),
            cache: HashMap::new(),
            main_id: 0,
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
            kind: m.kind,
            symbols: m.symbols,
            namespaces: m.namespaces,
        });

        let module = &self.modules[id];
        self.cache.insert(module.modpath.path().to_owned(), id);

        if module.modpath.path() == "main" {
            self.main_id = id;
        }

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

    /// Get main module
    pub fn main(&self) -> &Module {
        // main module should always exist if used
        self.get(self.main_id).expect("no main module")
    }

    pub fn modules(&self) -> &Vec<Module> {
        &self.modules
    }
}
