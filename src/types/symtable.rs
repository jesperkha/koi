use std::collections::HashMap;

// TODO: move to util?

/// The VarTable handles all name to type mappings for local and scoped variables.
pub struct VarTable<T> {
    /// A stack of scopes. Always has at least one base scope.
    scopes: Vec<HashMap<String, T>>,
}

impl<T> VarTable<T> {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
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

    /// Bind a name to T in the current scope. Return true if bind
    /// is ok (name was not already declared).
    pub fn bind(&mut self, name: String, t: T) -> bool {
        self.scopes.last_mut().unwrap().insert(name, t).is_none()
    }

    /// Look up a name starting from the innermost scope and iterating outwards.
    pub fn get(&self, name: &String) -> Option<&T> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t);
            }
        }
        None
    }

    /// Clear table
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.scopes.push(HashMap::new());
    }
}
