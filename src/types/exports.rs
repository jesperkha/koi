use std::collections::HashMap;

use crate::types::{TypeContext, TypeId, TypeKind};

pub enum Export {
    Function(FuncExport),
}

pub struct FuncExport {
    pub name: String,
    pub args: Vec<TypeKind>,
    pub ret: TypeKind,
}

pub struct Exports {
    symbols: HashMap<String, Export>,
}

impl Exports {
    pub fn new() -> Self {
        Exports {
            symbols: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, e: Export) {
        self.symbols.insert(name, e);
    }

    /// Import all symbols in this export into given TypeContext, declaring
    /// them as global symbols.
    pub fn import_into_ctx(&self, ctx: &mut TypeContext) {
        for (name, sym) in self.symbols.iter() {
            match sym {
                Export::Function(f) => {
                    let arg_ids: Vec<TypeId> = f
                        .args
                        .iter()
                        .map(|arg| ctx.get_or_intern(arg.clone()))
                        .collect();
                    let ret_id = ctx.get_or_intern(f.ret.clone());
                    let id = ctx.get_or_intern(TypeKind::Function(arg_ids, ret_id));
                    ctx.set_symbol(name.clone(), id, false);
                }
            }
        }
    }
}
