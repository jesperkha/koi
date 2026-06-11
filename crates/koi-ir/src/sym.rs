use std::collections::HashMap;

use crate::{ConstId, RValue};

pub struct SymTracker {
    tbl: HashMap<String, ConstId>,
    params: HashMap<String, usize>,
    curid: usize,
    curparam: usize,
}

impl Default for SymTracker {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn set(&mut self, t: String) -> ConstId {
        let id = self.curid;
        self.tbl.insert(t, id);
        self.curid += 1;
        id
    }

    pub fn next_const_id(&mut self) -> ConstId {
        let id = self.curid;
        self.curid += 1;
        id
    }

    pub fn set_param(&mut self, s: String) {
        self.params.insert(s, self.curparam);
        self.curparam += 1;
    }

    pub fn get(&self, s: &str) -> RValue {
        if let Some(v) = self.tbl.get(s).copied() {
            RValue::Const(v)
        } else {
            RValue::Param(
                *self
                    .params
                    .get(s)
                    .unwrap_or_else(|| panic!("tried to read undeclared name: {}", s)),
            )
        }
    }

    pub fn new_function_context(&mut self) {
        self.params.clear();
        self.tbl.clear();
        self.curid = 0;
        self.curparam = 0;
    }
}
