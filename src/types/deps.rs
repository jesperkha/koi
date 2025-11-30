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

    pub fn add(&mut self, name: String, ty: TypeKind) {
        self.symbols
            .insert(name, ty)
            .map(|_| panic!("duplicate symbol"));
    }

    pub fn get(&self, name: &str) -> Option<&TypeKind> {
        self.symbols.get(name)
    }
}

pub enum DependencyKind {
    User,
    Stdlib,
    ThirdParty,
}

pub struct Dependency {
    kind: DependencyKind,
    exports: Exports,
}

impl Dependency {
    pub fn user(exports: Exports) -> Self {
        Self {
            exports,
            kind: DependencyKind::User,
        }
    }

    pub fn exports(&self) -> &Exports {
        &self.exports
    }
}

pub struct Deps {
    dependencies: HashMap<String, Dependency>,
}

impl Deps {
    pub fn empty() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    pub fn with_stdlib() -> Self {
        Self::empty()
    }

    pub fn add(&mut self, name: String, dep: Dependency) {
        self.dependencies.insert(name, dep);
    }

    pub fn get(&self, name: &str) -> Option<&Dependency> {
        self.dependencies.get(name)
    }
}
