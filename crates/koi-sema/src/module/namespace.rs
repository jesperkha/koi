use std::collections::HashMap;

use crate::module::{Module, ModulePath, SymbolId};

pub struct Namespace {
    name: String,
    modpath: ModulePath,
    symbols: HashMap<String, SymbolId>,
}

impl Namespace {
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn modpath(&self) -> &ModulePath {
        &self.modpath
    }

    pub fn get(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }
}

pub struct NamespaceList {
    ns: HashMap<String, Namespace>,
}

impl Default for NamespaceList {
    fn default() -> Self {
        Self::new()
    }
}

impl NamespaceList {
    pub fn new() -> Self {
        Self { ns: HashMap::new() }
    }

    pub fn add(&mut self, ns: Namespace) -> Result<(), String> {
        self.ns.insert(ns.name.clone(), ns).map_or(Ok(()), |ns| {
            Err(format!("duplicate namespace '{}'", ns.name))
        })
    }

    pub fn get(&self, name: &str) -> Result<&Namespace, String> {
        self.ns.get(name).ok_or("not declared".to_string())
    }
}
