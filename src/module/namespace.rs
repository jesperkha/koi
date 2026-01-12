use std::collections::HashMap;

use crate::module::{Module, ModulePath, Symbol, SymbolList};

pub struct Namespace {
    /// Name of namespace in code (may be different from module name if aliased).
    name: String,
    modpath: ModulePath,
    symbols: SymbolList,
}

impl Namespace {
    pub fn new(name: String, module: &Module) -> Self {
        let mut ns = Namespace {
            name,
            modpath: module.modpath.clone(),
            symbols: SymbolList::new(),
        };

        for (_, sym) in module.exports() {
            let _ = ns.symbols.add(sym.clone());
        }

        ns
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn modpath(&self) -> &ModulePath {
        &self.modpath
    }

    pub fn add(&mut self, sym: Symbol) -> Result<(), String> {
        self.symbols.add(sym)
    }

    pub fn get(&self, name: &str) -> Result<&Symbol, String> {
        self.symbols.get(name)
    }
}

pub struct NamespaceList {
    ns: HashMap<String, Namespace>,
}

impl NamespaceList {
    pub fn new() -> Self {
        Self { ns: HashMap::new() }
    }

    pub fn add(&mut self, ns: Namespace) -> Result<(), String> {
        self.ns
            .insert(ns.name.clone(), ns)
            .map_or(Ok(()), |_| Err(format!("already declared")))
    }

    pub fn get(&self, name: &str) -> Result<&Namespace, String> {
        self.ns
            .get(name)
            .map_or(Err("not declared".to_string()), |s| Ok(s))
    }
}
