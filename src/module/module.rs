use std::{collections::HashMap, hash::Hash};

use crate::{
    module::{NamespaceList, Symbol, SymbolList, SymbolOrigin},
    types::TypedAst,
};

pub enum ModuleKind {
    Stdlib,
    User,
    Package,
}

pub type ModuleId = usize;

pub fn invalid_mod_id() -> ModuleId {
    return usize::MAX;
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
    ///
    /// ```
    /// use koi::module::ModulePath;
    ///
    /// let modpath = ModulePath::new_str("std.io");
    /// assert!(modpath.is_stdlib());
    ///
    /// let modpath2 = ModulePath::new_stdlib("foo");
    /// assert!(modpath2.is_stdlib());
    /// ```
    pub fn is_stdlib(&self) -> bool {
        self.0.starts_with("std.")
    }

    /// Get only the module name (the last identifier of the path).
    ///
    /// ```
    /// use koi::module::ModulePath;
    ///
    /// let modpath = ModulePath::new_str("app.foo.bar");
    /// assert_eq!(modpath.name(), "bar");
    ///
    /// let modpath2 = ModulePath::new_str("main");
    /// assert_eq!(modpath2.name(), "main");
    /// ```
    pub fn name(&self) -> &str {
        &self.0.split(".").last().unwrap() // asserted
    }

    /// Get the first part of the module path.
    ///
    /// ```
    /// use koi::module::ModulePath;
    ///
    /// let modpath = ModulePath::new_str("app.foo.bar");
    /// assert_eq!(modpath.first(), "app");
    /// ```
    pub fn first(&self) -> &str {
        &self.0.split(".").next().unwrap() // asserted
    }

    /// Get the full module path.
    pub fn path(&self) -> &str {
        &self.0
    }

    /// Get the module path with underscore (_) separators instead of period (.)
    ///
    /// ```
    /// use koi::module::ModulePath;
    ///
    /// let modpath = ModulePath::new_str("app.foo.bar");
    /// assert_eq!(modpath.path_underscore(), "app_foo_bar");
    /// ```
    pub fn path_underscore(&self) -> String {
        String::from(&self.0).replace(".", "_")
    }
}

pub struct CreateModule {
    pub modpath: ModulePath,
    pub filepath: String,
    pub ast: TypedAst,
    pub kind: ModuleKind,
    pub symbols: SymbolList,
    pub namespaces: NamespaceList,
    pub is_header: bool,
}

/// A Module is a self-contained compilation unit. It contains the combined
/// typed AST of all files in the module and all exported symbols.
pub struct Module {
    pub id: ModuleId,
    /// The id of this modules parent. 0 means this is root.
    pub parent: ModuleId,
    /// The module path, eg. app.some.mod
    /// The module name can be fetched from the module path.
    pub modpath: ModulePath,
    /// The relative path from src to this module.
    pub path: String,
    /// The fully typed AST generated from files in this module.
    pub ast: TypedAst,
    /// What type of module this is.
    pub kind: ModuleKind,
    /// List of symbols declared and used within this module.
    pub symbols: SymbolList,
    /// List of namespaces imported into this module.
    pub namespaces: NamespaceList,
    /// Is this module a header file? If true, do not build it.
    pub is_header: bool,
}

impl Module {
    pub fn name(&self) -> &str {
        self.modpath.name()
    }

    /// Reports whether this module should be built (produce IR/codegen) or not.
    pub fn should_be_built(&self) -> bool {
        !self.is_header
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

pub struct ModuleGraph {
    modules: Vec<Module>,
    /// Indecies in modules vec
    cache: HashMap<String, ModuleId>,
    /// id of main module
    main_id: ModuleId,
}

impl ModuleGraph {
    pub fn new() -> Self {
        ModuleGraph {
            modules: Vec::new(),
            cache: HashMap::new(),
            main_id: 0,
        }
    }

    /// Create a new module and add it to the graph.
    pub fn add(&mut self, m: CreateModule, parent: ModuleId) -> &Module {
        let id = self.modules.len();
        self.modules.push(Module {
            id,
            parent,
            modpath: m.modpath,
            path: m.filepath,
            ast: m.ast,
            kind: m.kind,
            is_header: m.is_header,
            symbols: m.symbols,
            namespaces: m.namespaces,
        });

        let module = &self.modules[id];
        self.cache.insert(module.modpath.path().to_owned(), id);

        if module.modpath.path() == "main" {
            self.main_id = id;
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

    /// Get main module
    pub fn main(&self) -> &Module {
        // main module should always exist if used
        self.get(self.main_id).expect("no main module")
    }

    pub fn modules(&self) -> &Vec<Module> {
        &self.modules
    }
}
