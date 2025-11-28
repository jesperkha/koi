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

    // /// Import all symbols in this export into given TypeContext as a namespace.
    // pub fn import_namespace_into_ctx(
    //     &self,
    //     namespace_name: String,
    //     ctx: &mut TypeContext,
    // ) -> Result<(), String> {
    //     let mut ns = Namespace::new(namespace_name);

    //     for (name, sym) in self.symbols.iter() {
    //         let id = self.intern_symbol(sym, ctx);
    //         ns.add(name.clone(), id);
    //     }

    //     ctx.add_namespace(ns)
    // }

    // /// Import the given symbols from this export into given context. Returns error on any name conflicts.
    // /// Declares all symbols globally as if defined in the context itself.
    // pub fn import_named_symbols_into_ctx(
    //     &self,
    //     symbol_names: &[&str],
    //     ctx: &mut TypeContext,
    // ) -> Result<(), String> {
    //     for name in symbol_names {
    //         match self.get(name) {
    //             Some(export) => {
    //                 let id = self.intern_symbol(export, ctx);
    //                 ctx.set_symbol(String::from(*name), id, false);
    //             }
    //             None => return Err(format!("no exported symbol '{}'", name)),
    //         }
    //     }

    //     Ok(())
    // }
}
