use std::collections::HashMap;

use crate::types::TypeId;

pub struct Namespace {
    pub name: String,
    pub symbols: HashMap<String, TypeId>,
}

impl Namespace {
    pub fn new(name: String) -> Self {
        Namespace {
            name,
            symbols: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, ty: TypeId) {
        self.symbols.insert(name, ty);
    }

    pub fn get(&mut self, name: String) -> Result<TypeId, String> {
        self.symbols
            .get(&name)
            .map_or(Err("not declared".to_string()), |t| Ok(*t))
    }
}
