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
}

impl Symbol {
    /// If the symbol is extern (resolved at link time).
    /// If this is true, then 'link_name' should be the same as 'name'.
    pub fn is_extern(&self) -> bool {
        matches!(self.origin, SymbolOrigin::Extern)
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
            SymbolKind::Function {
                is_inline,
                is_naked,
            } => {
                if *is_inline {
                    specs.push("inline");
                }
                if *is_naked {
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

#[derive(Clone, Debug)]
pub enum SymbolOrigin {
    Module {
        /// Module path of module origin
        modpath: ModulePath,
        /// Position of symbol declaration.
        pos: Pos,
        /// The filename where the symbol was declared.
        filename: String,
    },
    Library(ModulePath),
    Extern,
}

impl fmt::Display for SymbolOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SymbolOrigin::Module { modpath, .. } => format!("module({})", modpath),
                SymbolOrigin::Extern => "extern".to_string(),
                SymbolOrigin::Library(modpath) => format!("library({})", modpath),
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SymbolKind {
    Function {
        /// If the function body should be inlined.
        is_inline: bool,
        /// If the function body should be naked (no entry/exit protocol or additional
        /// code added by the compiler).
        is_naked: bool,
    },
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SymbolKind::Function { .. } => "function",
            }
        )
    }
}

/// ModuleSymbol represents a symbol imported or declared in a module.
pub struct ModuleSymbol {
    pub id: SymbolId,
    pub kind: ModuleSymbolKind,
    pub exported: bool,
}

pub enum ModuleSymbolKind {
    /// Module symbols are symbols declared in this module.
    Module,
    /// Imported symbols are any extern or package imported symbols.
    Imported,
}

pub struct SymbolList {
    symbols: HashMap<String, ModuleSymbol>,
}

impl Default for SymbolList {
    fn default() -> Self {
        Self::new()
    }
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
            .map_or(Ok(()), |_| Err("already declared".to_string()))
    }

    pub fn get(&self, name: &str) -> Result<&ModuleSymbol, String> {
        self.symbols.get(name).ok_or("not declared".to_string())
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
