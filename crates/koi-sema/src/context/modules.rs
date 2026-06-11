use std::collections::HashMap;

use tracing::debug;

use crate::module::{ImportPath, Module, ModuleId, ModuleKind, ModulePath, SymbolList};

pub const INVALID_MOD_ID: ModuleId = usize::MAX;

pub struct CreateModule {
    pub modpath: ModulePath,
    pub kind: ModuleKind,
    pub symbols: SymbolList,
}

pub struct ModuleInterner {
    modules: Vec<Module>,
    cache: HashMap<String, ModuleId>,
    main_id: Option<ModuleId>,
}

impl Default for ModuleInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleInterner {
    pub fn new() -> Self {
        ModuleInterner {
            modules: Vec::new(),
            cache: HashMap::new(),
            main_id: None,
        }
    }

    pub fn add(&mut self, m: CreateModule) -> ModuleId {
        let id = self.modules.len();

        let key = match &m.kind {
            ModuleKind::Source { .. } => m.modpath.path().to_string(),
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
        });

        id
    }

    pub fn get(&self, id: ModuleId) -> &Module {
        assert!(id != INVALID_MOD_ID, "invalid mod id");
        &self.modules[id]
    }

    pub fn try_get(&self, id: ModuleId) -> Option<&Module> {
        if id == INVALID_MOD_ID {
            return None;
        }
        self.modules.get(id)
    }

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
                Err("could not resolve module import".to_string())
            },
            |id| Ok(&self.modules[*id]),
        )
    }

    pub fn main(&self) -> Option<&Module> {
        self.main_id.map(|id| self.get(id))
    }

    pub fn modules(&self) -> &[Module] {
        &self.modules
    }
}
