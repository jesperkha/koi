use std::collections::HashMap;

use crate::{
    module::{Symbol, SymbolList},
    types::{TypeContext, TypeKind},
};

pub struct ExportedSymbol {
    pub symbol: Symbol,
    pub kind: TypeKind,
}

pub struct Exports {
    exports: HashMap<String, ExportedSymbol>,
}

impl Exports {
    pub fn extract(ctx: &TypeContext, syms: &SymbolList) -> Self {
        let exports = syms
            .symbols()
            .iter()
            .filter(|s| s.1.is_exported)
            .map(|s| {
                (
                    s.0.clone(),
                    ExportedSymbol {
                        symbol: s.1.clone(),
                        kind: ctx.lookup(s.1.ty).kind.clone(),
                    },
                )
            })
            .collect();

        Exports { exports }
    }

    pub fn get_type(&self, name: &str) -> Option<&TypeKind> {
        self.exports.get(name).map(|s| &s.kind)
    }

    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        self.exports.get(name).map(|s| &s.symbol)
    }

    pub fn get(&self, name: &str) -> Option<&ExportedSymbol> {
        self.exports.get(name)
    }

    pub fn symbols(&self) -> &HashMap<String, ExportedSymbol> {
        &self.exports
    }
}
