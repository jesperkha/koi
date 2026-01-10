use std::{collections::HashMap, hash::Hash};

use crate::{
    error::Error,
    module::{Exports, ModulePath},
    types::{TypeContext, TypeId},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    /// Name of namespace in code (may be different from module name if aliased).
    pub name: String,
    pub modpath: ModulePath,
    pub symbols: HashMap<String, TypeId>,
}

impl Namespace {
    pub fn new(
        name: String,
        modpath: ModulePath,
        exports: &Exports,
        ctx: &mut TypeContext,
    ) -> Self {
        let mut ns = Namespace {
            name,
            modpath,
            symbols: HashMap::new(),
        };

        for (name, sym) in exports.symbols() {
            let id = ctx.get_or_intern(sym.kind.clone());
            ns.symbols.insert(name.to_string(), id);
        }

        ns
    }

    pub fn module_name(&self) -> &str {
        self.modpath.name()
    }

    pub fn module_path(&self) -> &str {
        self.modpath.path()
    }
}

impl Hash for Namespace {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.modpath.path().hash(state);
    }
}

pub struct NamespaceList {
    namespaces: HashMap<String, Namespace>,
}

impl NamespaceList {
    pub fn new() -> Self {
        Self {
            namespaces: HashMap::new(),
        }
    }

    pub fn add(&mut self, ns: Namespace) -> Result<(), String> {
        self.namespaces
            .insert(ns.name.clone(), ns)
            .map_or(Ok(()), |_| Err(String::from("already declared")))
    }

    pub fn get(&self, name: &str) -> Result<&Namespace, String> {
        self.namespaces
            .get(name)
            .map_or(Err(String::from("not declared")), |ns| Ok(ns))
    }
}
