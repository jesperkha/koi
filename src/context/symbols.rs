use crate::{
    module::{Symbol, SymbolId, SymbolKind, SymbolOrigin},
    types::TypeId,
};

pub const INVALID_SYMBOL_ID: SymbolId = usize::MAX;

#[derive(Debug)]
pub struct CreateSymbol {
    pub kind: SymbolKind,
    pub ty: TypeId,
    pub name: String,
    pub origin: SymbolOrigin,
    pub is_exported: bool,
    pub no_mangle: bool,
}

pub struct SymbolInterner {
    symbols: Vec<Symbol>,
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
            origin: symbol.origin,
            is_exported: symbol.is_exported,
            no_mangle: symbol.no_mangle,
        };

        let id = symbol.id;
        self.symbols.push(symbol);
        id
    }

    /// Get a symbol by ID. Panics if the ID is invalid.
    /// Use this when you know the ID came from this interner.
    pub fn get(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id]
    }

    /// Try to get a symbol by ID, returning None if the ID is out of bounds.
    pub fn try_get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id)
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// Create a string dump of all symbols in this module.
    pub fn dump(&self, module: &str) -> String {
        let mut s = String::new();
        s += &format!("| Symbols in {}\n", module);
        s += &format!("| ----------------------\n");
        for sym in &self.symbols {
            s += &format!("| {:<20} {}\n", sym.name, sym)
        }
        s
    }
}
