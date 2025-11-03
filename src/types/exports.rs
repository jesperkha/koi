use std::collections::HashMap;

use crate::types::{Namespace, TypeContext, TypeId, TypeKind};

pub enum Export {
    Function(FuncExport),
}

pub struct FuncExport {
    pub name: String,
    pub args: Vec<TypeKind>,
    pub ret: TypeKind,
}

pub struct Exports {
    pub pkgname: String,
    symbols: HashMap<String, Export>,
}

impl Exports {
    pub fn new(pkgname: String) -> Self {
        Exports {
            pkgname,
            symbols: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, e: Export) {
        self.symbols.insert(name, e);
    }

    fn get(&self, name: &str) -> Option<&Export> {
        self.symbols.get(name)
    }

    /// Import all symbols in this export into given TypeContext as a namespace.
    pub fn import_namespace_into_ctx(
        &self,
        namespace_name: String,
        ctx: &mut TypeContext,
    ) -> Result<(), String> {
        let mut ns = Namespace::new(namespace_name);

        for (name, sym) in self.symbols.iter() {
            let id = self.intern_symbol(sym, ctx);
            ns.add(name.clone(), id);
        }

        ctx.add_namespace(ns)
    }

    /// Import the given symbols from this export into given context. Returns error on any name conflicts.
    /// Declares all symbols globally as if defined in the context itself.
    pub fn import_named_symbols_into_ctx(
        &self,
        symbol_names: &[&str],
        ctx: &mut TypeContext,
    ) -> Result<(), String> {
        for name in symbol_names {
            match self.get(name) {
                Some(export) => {
                    let id = self.intern_symbol(export, ctx);
                    ctx.set_symbol(String::from(*name), id, false);
                }
                None => return Err(format!("no exported symbol '{}'", name)),
            }
        }

        Ok(())
    }

    fn intern_symbol(&self, export: &Export, ctx: &mut TypeContext) -> TypeId {
        match export {
            Export::Function(f) => {
                let arg_ids: Vec<TypeId> = f
                    .args
                    .iter()
                    .map(|arg| ctx.get_or_intern(arg.clone()))
                    .collect();

                let ret_id = ctx.get_or_intern(f.ret.clone());
                ctx.get_or_intern(TypeKind::Function(arg_ids, ret_id))
            }
        }
    }
}
