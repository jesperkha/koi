use std::collections::HashMap;

use crate::module::{Symbol, SymbolList};

pub struct Exports {
    exports: HashMap<String, Symbol>,
}

impl Exports {
    pub fn extract(syms: &SymbolList) -> Self {
        let exports = syms
            .symbols()
            .iter()
            .filter(|s| s.1.is_exported)
            .map(|s| (s.0.clone(), s.1.clone()))
            .collect();

        Exports { exports }
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.exports.get(name)
    }

    pub fn symbols(&self) -> &HashMap<String, Symbol> {
        &self.exports
    }
}
