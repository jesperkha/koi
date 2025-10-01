use std::collections::HashMap;

use crate::{token::Token, types::TypeId};

/// The SymTable manages all symbol/type mappings in a file.
/// The mappings are only used when type checking.
pub struct SymTable {
    /// A stack of scopes. Always has at least one base scope.
    scopes: Vec<HashMap<String, TypeId>>,
    /// Map of global type declarations.
    type_decls: HashMap<String, TypeId>,
}

impl SymTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            type_decls: HashMap::new(),
        }
    }

    /// Push a new empty scope
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope. Panics if base is popped.
    pub fn pop_scope(&mut self) {
        assert!(self.scopes.len() > 1, "attempted to pop base scope");
        self.scopes.pop();
    }

    /// Bind a name to a type in the current scope. Return true if bind
    /// is ok (name was not already declared).
    pub fn bind(&mut self, name: &Token, ty: TypeId) -> bool {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), ty)
            .is_none()
    }

    /// Look up a name starting from the innermost scope outward.
    pub fn get_symbol(&self, name: &String) -> Option<TypeId> {
        for scope in self.scopes.iter().rev() {
            if let Some(&ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Declare a global user type.
    pub fn declare(&mut self, name: String, ty: TypeId) {
        self.type_decls.insert(name, ty);
    }

    /// Get a declared type.
    pub fn get_type(&self, name: &Token) -> Option<TypeId> {
        self.type_decls.get(&name.to_string()).copied()
    }
}
