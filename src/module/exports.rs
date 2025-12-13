use std::collections::HashMap;

use crate::types::TypeKind;

pub struct Exports {
    symbols: HashMap<String, TypeKind>,
}

impl Exports {
    pub fn new() -> Self {
        Exports {
            symbols: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, kind: TypeKind) {
        self.symbols.insert(name, kind);
    }

    pub fn get(&self, name: &str) -> Option<&TypeKind> {
        self.symbols.get(name)
    }

    pub fn symbols(&self) -> &HashMap<String, TypeKind> {
        &self.symbols
    }
}
