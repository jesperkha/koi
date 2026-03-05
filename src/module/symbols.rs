use core::fmt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{ast::Pos, context::Context, module::ModulePath, types::TypeId};

pub type SymbolId = usize;

#[derive(Clone, Debug)]
pub struct Symbol {
    /// Unique identifier
    pub id: SymbolId,
    /// Symbol kind contains additional, more specific, symbol information.
    pub kind: SymbolKind,
    /// The symbols type.
    pub ty: TypeId,
    /// The symbol name as it was declared.
    pub name: String,
    /// Where the symbol originates from.
    pub origin: SymbolOrigin,
    /// If this symbol is exported from its origin module.
    pub is_exported: bool,
    /// True if the symbol name should not be mangled when linking
    pub no_mangle: bool,
    /// Position of symbol declaration.
    pub pos: Pos,
    /// The filename where the symbol was declared.
    pub filename: String,
}

impl Symbol {
    /// If the symbol is extern (resolved at link time).
    /// If this is true, then 'link_name' should be the same as 'name'.
    pub fn is_extern(&self) -> bool {
        matches!(self.origin, SymbolOrigin::Extern(_))
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut specs = vec![];
        if self.is_exported {
            specs.push("exported");
        }
        if !self.no_mangle {
            specs.push("mangled");
        }
        if self.is_extern() {
            specs.push("extern");
        }
        match &self.kind {
            SymbolKind::Function(func) => {
                if func.is_inline {
                    specs.push("inline");
                }
                if func.is_naked {
                    specs.push("naked");
                }
            }
        }
        write!(
            f,
            "[{} {} origin={} typeid={} {}]",
            self.kind,
            self.name,
            self.origin,
            self.ty,
            specs.join(" "),
        )
    }
}

// TODO: use module id or something else
#[derive(Clone, Debug)]
pub enum SymbolOrigin {
    Module(ModulePath),
    Extern(ModulePath), // Contains origin of declaration
}

impl fmt::Display for SymbolOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SymbolOrigin::Module(modpath) => format!("module({})", modpath),
                SymbolOrigin::Extern(modpath) => format!("extern({})", modpath),
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SymbolKind {
    Function(FuncSymbol),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FuncSymbol {
    /// If the function body should be inlined.
    pub is_inline: bool,
    /// If the function body should be naked (no entry/exit protocol or additional
    /// code added by the compiler).
    pub is_naked: bool,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SymbolKind::Function(_) => "function",
            }
        )
    }
}

pub struct ModuleSymbol {
    pub id: SymbolId,
    /// Should this symbol be exported? This should only be true if the symbol
    /// is marked as public AND the symbol originates from this module.
    pub exported: bool,
}

pub struct SymbolList {
    symbols: HashMap<String, ModuleSymbol>,
}

impl SymbolList {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, symbol: ModuleSymbol) -> Result<(), String> {
        self.symbols
            .insert(name, symbol)
            .map_or(Ok(()), |_| Err(format!("already declared")))
    }

    pub fn get(&self, name: &str) -> Result<&ModuleSymbol, String> {
        self.symbols
            .get(name)
            .map_or(Err(format!("not declared")), |s| Ok(s))
    }

    pub fn symbols(&self) -> &HashMap<String, ModuleSymbol> {
        &self.symbols
    }

    pub fn dump(&self, ctx: &Context, filepath: &str) -> String {
        let mut s = String::new();
        s += &format!("Symbols in {}\n", filepath);
        s += "-----------------------------------\n";

        for (name, modsym) in &self.symbols {
            let symbol = ctx.symbols.get(modsym.id);
            s += &format!("{:<20}{}\n", name, symbol);
        }

        s
    }
}

impl From<HashMap<String, ModuleSymbol>> for SymbolList {
    fn from(symbols: HashMap<String, ModuleSymbol>) -> Self {
        Self { symbols }
    }
}
