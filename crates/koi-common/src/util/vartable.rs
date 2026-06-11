use std::collections::HashMap;

pub struct VarTable<T> {
    scopes: Vec<HashMap<String, T>>,
}

impl<T> Default for VarTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> VarTable<T> {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        assert!(self.scopes.len() > 1, "attempted to pop base scope");
        self.scopes.pop();
    }

    pub fn bind(&mut self, name: String, t: T) -> bool {
        if self.cur_scope().get(&name).is_some() {
            return false;
        }
        self.scopes.last_mut().unwrap().insert(name, t).is_none()
    }

    fn cur_scope(&self) -> &HashMap<String, T> {
        self.scopes.last().unwrap()
    }

    pub fn get(&self, name: &str) -> Option<&T> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t);
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.scopes.clear();
        self.scopes.push(HashMap::new());
    }
}
