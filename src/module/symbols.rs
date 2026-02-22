use core::fmt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{ast::Pos, module::ModulePath, types::TypeId};

#[derive(Clone, Debug)]
pub struct Symbol {
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
    /// True if the symbol name should not be mangled (link_name).
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

    // TODO: (doing) some form of prefix for libaries
    // to not collide accidently with user code
    // 1. In emit, take in the project type and prefix all user symbols
    // 2. In match below, add the library prefix

    /// The mangled link name (prefixed with module path etc).
    /// For any symbol named 'main' it will return 'main'.
    /// For any extern symbol it will return the unaltered name.
    /// If no_mangle is true the unaltered symbol name is returned.
    pub fn link_name(&self) -> String {
        if self.name == "main" {
            return String::from("main");
        }
        if self.no_mangle {
            return self.name.clone();
        }
        match &self.origin {
            SymbolOrigin::Module(modpath) => {
                format!("_{}_{}", modpath.path_underscore(), self.name)
            }
            SymbolOrigin::Extern(_) => self.name.clone(),
            SymbolOrigin::Library(_) => todo!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SymbolOrigin {
    Module(ModulePath),
    Extern(ModulePath), // Contains origin of declaration
    Library(String),    // Contains the library name
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

pub struct SymbolList {
    symbols: HashMap<String, Symbol>,
}

impl From<Vec<Symbol>> for SymbolList {
    fn from(symbols: Vec<Symbol>) -> Self {
        Self {
            symbols: symbols.into_iter().map(|s| (s.name.clone(), s)).collect(),
        }
    }
}

impl SymbolList {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn add(&mut self, sym: Symbol) -> Result<(), String> {
        if self.symbols.contains_key(&sym.name) {
            return Err("already declared".to_string());
        }
        self.symbols.insert(sym.name.clone(), sym);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&Symbol, String> {
        self.symbols
            .get(name)
            .map_or(Err("not declared".to_string()), |s| Ok(s))
    }

    pub fn symbols(&self) -> &HashMap<String, Symbol> {
        &self.symbols
    }

    /// Create a string dump of all symbols in this module.
    pub fn dump(&self, module: &str) -> String {
        let mut s = String::new();
        s += &format!("| Symbols in {}\n", module);
        s += &format!("| ----------------------\n");
        for (name, sym) in &self.symbols {
            s += &format!("| {:<10} {}\n", name, sym)
        }
        s
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Symbol(name={}, kind={}, origin={}, typeid={}, exported={}, mangled={}, extern={})",
            self.name,
            self.kind,
            self.origin,
            self.ty,
            self.is_exported,
            !self.no_mangle,
            self.is_extern(),
        )
    }
}

impl fmt::Display for SymbolOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SymbolOrigin::Module(modpath) => format!("module({})", modpath),
                SymbolOrigin::Extern(modpath) => format!("extern({})", modpath),
                SymbolOrigin::Library(libname) => format!("library({})", libname),
            }
        )
    }
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SymbolKind::Function(s) =>
                    format!("Func(inline={}, naked={})", s.is_inline, s.is_naked),
            }
        )
    }
}
