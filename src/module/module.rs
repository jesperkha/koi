use std::{collections::HashMap, hash::Hash};

use crate::{
    ast::ImportNode,
    module::{NamespaceList, Symbol, SymbolList, SymbolOrigin},
    types::TypedAst,
};

pub type ModuleId = usize;

pub fn invalid_mod_id() -> ModuleId {
    return usize::MAX;
}

/// A Module is a self-contained compilation unit. It contains the combined
/// typed AST of all files in the module and all exported symbols.
pub struct Module {
    pub id: ModuleId,
    /// What type of module this is.
    pub kind: ModuleKind,
    /// The module path, eg. app.some.mod
    /// The module name can be fetched from the module path.
    pub modpath: ModulePath,
    /// List of symbols declared and used within this module.
    pub symbols: SymbolList,
    /// List of modules this module depends on.
    pub deps: Vec<ModuleId>,
}

pub enum ModuleKind {
    /// Source module are created from the source code of the current project.
    /// These modules are built into the final executable/library.
    Source(SourceModule),

    /// External modules are external libraries, such as the standard library,
    /// and are skipped when building, as they are pre-compiled.
    External(ExternalModule),
}

pub struct SourceModule {
    /// The relative path from src to this module.
    /// For package modules this is the filepath to the linkable object file.
    pub path: String,
    /// The fully typed AST generated from files in this module.
    pub ast: TypedAst,
    /// List of namespaces imported into this module.
    pub namespaces: NamespaceList,
}

pub struct ExternalModule {
    /// Full filepath to this modules header file.
    pub header_path: String,
    /// Full fileapth to this modules archive file.
    pub archive_path: String,
}

impl Module {
    pub fn name(&self) -> &str {
        self.modpath.name()
    }

    pub fn is_main(&self) -> bool {
        self.modpath.path() == "main"
    }

    /// Reports whether this module should be built (produce IR/codegen) or not.
    pub fn should_be_built(&self) -> bool {
        matches!(self.kind, ModuleKind::Source(_))
    }

    /// Collect all exported symbols from this module.
    pub fn exports(&self) -> HashMap<&String, &Symbol> {
        self.symbols
            .symbols()
            .iter()
            .filter(|s| {
                // Is it exported?
                if !s.1.is_exported {
                    return false;
                }

                match &s.1.origin {
                    SymbolOrigin::Module(modpath) => &self.modpath == modpath,
                    SymbolOrigin::Extern(modpath) => &self.modpath == modpath,
                }
            })
            .collect::<_>()
    }
}

/// Module path wraps a string module path (app.foo.bar) and provides methods
/// to get the path itself or the module name (the last name in the path).
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ModulePath(String);

impl ModulePath {
    pub fn new(s: String) -> Self {
        assert!(!s.is_empty(), "cannot have empty module path");
        Self(s)
    }

    pub fn new_str(s: &str) -> Self {
        assert!(!s.is_empty(), "cannot have empty module path");
        Self(s.to_owned())
    }

    pub fn new_package(name: &str) -> Self {
        assert!(!name.is_empty(), "cannot have empty module path");
        Self(format!("lib.{}", name))
    }

    pub fn new_stdlib(name: &str) -> Self {
        assert!(!name.is_empty(), "cannot have empty module path");
        Self(format!("std.{}", name))
    }

    /// Check if this module path is part of the standard library.
    pub fn is_stdlib(&self) -> bool {
        self.0.starts_with("std.")
    }

    /// Get only the module name (the last identifier of the path).
    pub fn name(&self) -> &str {
        &self.0.split(".").last().unwrap() // asserted
    }

    /// Get the first part of the module path.
    pub fn first(&self) -> &str {
        &self.0.split(".").next().unwrap() // asserted
    }

    /// Get the full module path.
    pub fn path(&self) -> &str {
        &self.0
    }

    /// Get the module path with underscore (_) separators instead of period (.)
    pub fn path_underscore(&self) -> String {
        String::from(&self.0).replace(".", "_")
    }
}

impl From<&str> for ModulePath {
    fn from(s: &str) -> Self {
        ModulePath::new(s.to_string())
    }
}

impl From<String> for ModulePath {
    fn from(s: String) -> Self {
        ModulePath::new(s)
    }
}

impl From<&ImportNode> for ModulePath {
    fn from(import: &ImportNode) -> Self {
        ModulePath::new(
            import
                .names
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join("."),
        )
    }
}

pub struct CreateModule {
    pub modpath: ModulePath,
    pub kind: ModuleKind,
    pub symbols: SymbolList,
    pub deps: Vec<ModuleId>,
}

pub struct ModuleGraph {
    modules: Vec<Module>,
    /// Indecies in modules vec
    cache: HashMap<String, ModuleId>,
    /// id of main module
    main_id: Option<ModuleId>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        ModuleGraph {
            modules: Vec::new(),
            cache: HashMap::new(),
            main_id: None,
        }
    }

    /// Create a new module and add it to the graph.
    pub fn add(&mut self, m: CreateModule) -> &Module {
        let id = self.modules.len();
        self.modules.push(Module {
            id,
            modpath: m.modpath,
            symbols: m.symbols,
            kind: m.kind,
            deps: m.deps,
        });

        let module = &self.modules[id];
        self.cache.insert(module.modpath.path().to_owned(), id);

        if module.modpath.path() == "main" {
            self.main_id = Some(id);
        }

        module
    }

    pub fn get(&self, id: ModuleId) -> Option<&Module> {
        assert!(id != invalid_mod_id(), "invalid mod id");
        self.modules.get(id)
    }

    /// Resolve a module path to a Module, referenced from the internal array.
    pub fn resolve(&self, modpath: &ModulePath) -> Result<&Module, String> {
        self.cache
            .get(modpath.path())
            .map_or(Err(format!("could not resolve module path")), |id| {
                Ok(&self.modules[*id])
            })
    }

    /// Get main module if any
    pub fn main(&self) -> Option<&Module> {
        self.main_id.and_then(|id| self.get(id))
    }

    pub fn modules(&self) -> &Vec<Module> {
        &self.modules
    }
}
