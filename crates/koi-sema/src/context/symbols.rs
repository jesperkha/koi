use crate::{
    module::{Symbol, SymbolId, SymbolKind, SymbolOrigin},
    types::TypeId,
};

pub const INVALID_SYMBOL_ID: SymbolId = usize::MAX;

#[derive(Debug)]
pub struct CreateSymbol {
    pub name: String,
    pub alias: Option<String>,
    pub kind: SymbolKind,
    pub ty: TypeId,
    pub origin: SymbolOrigin,
    pub is_exported: bool,
    pub no_mangle: bool,
}

pub struct SymbolInterner {
    symbols: Vec<Symbol>,
}

impl Default for SymbolInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolInterner {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    pub fn add(&mut self, symbol: CreateSymbol) -> SymbolId {
        let symbol = Symbol {
            id: self.symbols.len(),
            kind: symbol.kind,
            ty: symbol.ty,
            name: symbol.name,
            alias: symbol.alias,
            origin: symbol.origin,
            is_exported: symbol.is_exported,
            no_mangle: symbol.no_mangle,
        };

        let id = symbol.id;
        self.symbols.push(symbol);
        id
    }

    pub fn get(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id]
    }

    pub fn try_get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id)
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    pub fn dump(&self, module: &str) -> String {
        let mut s = String::new();
        s += &format!("| Symbols in {}\n", module);
        s += "| ----------------------\n";
        for sym in &self.symbols {
            s += &format!("| {:<20} {}\n", sym.name, sym)
        }
        s
    }
}
