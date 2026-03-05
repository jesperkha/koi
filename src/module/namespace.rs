use std::collections::HashMap;

use crate::module::{Module, ModulePath, SymbolId};

pub struct Namespace {
    name: String,
    modpath: ModulePath,
    symbols: HashMap<String, SymbolId>,
}

impl Namespace {
    /// Create a new namespace from a module's exports.
    pub fn new(name: String, module: &Module) -> Self {
        Self {
            name,
            modpath: module.modpath.clone(),
            symbols: module
                .exports()
                .iter()
                .map(|(name, id)| ((*name).to_owned(), *id))
                .collect(),
        }
    }

    /// Name of namespace in code (may be different from module name if aliased).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The module path this namespace refers to.
    pub fn modpath(&self) -> &ModulePath {
        &self.modpath
    }

    /// Get a symbol from this namespace.
    pub fn get(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }
}

pub struct NamespaceList {
    ns: HashMap<String, Namespace>,
}

impl NamespaceList {
    pub fn new() -> Self {
        Self { ns: HashMap::new() }
    }

    /// Add a namespace to the list.
    pub fn add(&mut self, ns: Namespace) -> Result<(), String> {
        self.ns.insert(ns.name.clone(), ns).map_or(Ok(()), |ns| {
            Err(format!("duplicate namespace '{}'", ns.name))
        })
    }

    /// Get a namespace by name.
    pub fn get(&self, name: &str) -> Result<&Namespace, String> {
        self.ns
            .get(name)
            .map_or(Err("not declared".to_string()), |s| Ok(s))
    }
}
