use core::fmt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use koi_ast::Pos;

use crate::{context::Context, module::ModulePath, types::TypeId};

pub type SymbolId = usize;

#[derive(Clone, Debug)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub alias: Option<String>,
    pub kind: SymbolKind,
    pub ty: TypeId,
    pub origin: SymbolOrigin,
    pub is_exported: bool,
    pub no_mangle: bool,
}

impl Symbol {
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
            SymbolKind::Type => specs.push("type"),
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
    Intrinsic,
    Module {
        modpath: ModulePath,
        pos: Pos,
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
                SymbolOrigin::Intrinsic => "intrinsic".into(),
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SymbolKind {
    Function {
        is_inline: bool,
        is_naked: bool,
    },
    Type,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SymbolKind::Function { .. } => "function",
                SymbolKind::Type => "type",
            }
        )
    }
}

pub struct ModuleSymbol {
    pub id: SymbolId,
    pub kind: ModuleSymbolKind,
    pub exported: bool,
}

pub enum ModuleSymbolKind {
    Module,
    Imported,
}

pub struct SymbolList {
    aliases: HashMap<String, String>,
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
            aliases: HashMap::new(),
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

    pub fn set_alias(&mut self, from: String, to: String) {
        self.aliases.insert(from, to);
    }

    pub fn get_alias(&self, real_name: &String) -> Option<&String> {
        self.aliases.get(real_name)
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
        Self {
            symbols,
            aliases: HashMap::new(),
        }
    }
}
