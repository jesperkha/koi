use std::collections::HashMap;

use crate::ir::{ConstId, Value};

/// SymTracker keeps track of ids for named symbols in the current function
/// context. Setting a new name overrides the previous and thus provides SSA
/// ids for all variable and parameter names.
pub struct SymTracker {
    tbl: HashMap<String, ConstId>,
    params: HashMap<String, usize>,
    curid: usize,
    curparam: usize,
}

impl SymTracker {
    pub fn new() -> Self {
        Self {
            tbl: HashMap::new(),
            params: HashMap::new(),
            curid: 0,
            curparam: 0,
        }
    }

    /// Create new const id for a token, overriding any previous one.
    pub fn set(&mut self, t: String) -> ConstId {
        let id = self.curid;
        self.tbl.insert(t, id);
        self.curid += 1;
        id
    }

    /// Create new const id for a temporary value
    pub fn next(&mut self) -> ConstId {
        let id = self.curid;
        self.curid += 1;
        id
    }

    /// Create new parameter id for name in this context
    pub fn set_param(&mut self, s: String) {
        self.params.insert(s, self.curparam);
        self.curparam += 1;
    }

    /// Look up a name in the current context
    pub fn get(&self, s: &String) -> Value {
        if let Some(v) = self.tbl.get(s).map(|t| *t) {
            Value::Const(v)
        } else {
            Value::Param(
                *self
                    .params
                    .get(s)
                    .expect(&format!("tried to read undeclared name: {}", s)), // bug
            )
        }
    }

    /// Reset param register and symbol table
    pub fn new_function_context(&mut self) {
        self.params.clear();
        self.tbl.clear();
        self.curid = 0;
        self.curparam = 0;
    }
}
