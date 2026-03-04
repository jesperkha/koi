use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{ast::Pos, module::ModulePath, types::TypeId};

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
