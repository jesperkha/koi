use std::collections::HashMap;

use tracing::debug;

use crate::module::{ImportPath, Module, ModuleId, ModuleKind, ModulePath, SymbolList};

pub const INVALID_MOD_ID: ModuleId = usize::MAX;

pub struct CreateModule {
    pub modpath: ModulePath,
    pub kind: ModuleKind,
    pub symbols: SymbolList,
    pub deps: Vec<ModuleId>,
}

pub struct ModuleInterner {
    modules: Vec<Module>,
    /// Indecies in modules vec
    cache: HashMap<String, ModuleId>,
    /// id of main module
    main_id: Option<ModuleId>,
}

impl ModuleInterner {
    pub fn new() -> Self {
        ModuleInterner {
            modules: Vec::new(),
            cache: HashMap::new(),
            main_id: None,
        }
    }

    /// Create a new module and add it to the graph.
    pub fn add(&mut self, m: CreateModule) -> ModuleId {
        let id = self.modules.len();

        let key = match &m.kind {
            // For source modules the import path should have the prefix and package name removed
            // (eg. myapp.util -> util). This is purely for convenience.
            ModuleKind::Source { .. } => m.modpath.path().to_string(),
            // For external modules the full import path is used to preserve the prefix and package
            // name (eg. lib.mylib.foo).
            ModuleKind::External => m.modpath.import_path().to_string(),
        };

        if m.modpath.is_main() {
            self.main_id = Some(id);
        }

        self.cache.insert(key, id);
        self.modules.push(Module {
            id,
            modpath: m.modpath,
            symbols: m.symbols,
            kind: m.kind,
            deps: m.deps,
        });

        id
    }

    /// Get a module by ID. Panics if the ID is invalid.
    /// Use this when you know the ID came from this interner.
    pub fn get(&self, id: ModuleId) -> &Module {
        assert!(id != INVALID_MOD_ID, "invalid mod id");
        &self.modules[id]
    }

    /// Try to get a module by ID, returning None if out of bounds.
    pub fn try_get(&self, id: ModuleId) -> Option<&Module> {
        if id == INVALID_MOD_ID {
            return None;
        }
        self.modules.get(id)
    }

    /// Resolve a module path to a Module, referenced from the internal array.
    pub fn resolve(&self, impath: &ImportPath) -> Result<&Module, String> {
        self.cache.get(impath.path()).map_or_else(
            || {
                debug!("ImportPath={:?}", impath);
                debug!(
                    "Available=[{}]",
                    self.cache
                        .keys()
                        .map(|k| k.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                Err(format!("could not resolve module import"))
            },
            |id| Ok(&self.modules[*id]),
        )
    }

    /// Get main module if any
    pub fn main(&self) -> Option<&Module> {
        self.main_id.map(|id| self.get(id))
    }

    pub fn modules(&self) -> &[Module] {
        &self.modules
    }
}
