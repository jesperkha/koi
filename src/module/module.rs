use std::{collections::HashMap, fmt::Display, hash::Hash, path::PathBuf};

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
    // TODO: make separate list of symbols imported by name
    // to not make .exports() dependent on anything
    /// List of modules this module depends on.
    pub deps: Vec<ModuleId>,
}

pub enum ModuleKind {
    /// Source module are created from the source code of the current project.
    /// These modules are built into the final executable/library.
    Source(SourceModule),

    /// External modules are external libraries, such as the standard library,
    /// and are skipped when building, as they are pre-compiled.
    External,
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

impl Module {
    pub fn name(&self) -> &str {
        self.modpath.path() // always non-empty in usecases
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
pub struct ModulePath {
    // Examples values for the full module path lib.socket.common.util
    prefix: String,  // lib
    package: String, // socket
    path: String,    // common.util
    is_main: bool,
}

impl ModulePath {
    pub fn new(prefix: String, package: String, path: String) -> Self {
        if !prefix.is_empty() {
            assert!(
                !package.is_empty(),
                "cannot have empty package name if prefix is non-empty",
            );
        }
        if prefix.is_empty() && package.is_empty() && path.is_empty() {
            panic!("empty module path");
        }
        Self {
            prefix,
            package,
            path,
            is_main: false,
        }
    }

    pub fn to_main(self) -> Self {
        Self {
            prefix: self.prefix,
            package: self.package,
            path: self.path,
            is_main: true,
        }
    }

    /// Create new standard library module path
    pub fn to_std(self) -> ModulePath {
        ModulePath::new("std".into(), self.package, self.path)
    }

    /// Create new external library module path
    pub fn to_lib(self) -> ModulePath {
        ModulePath::new("lib".into(), self.package, self.path)
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn is_main(&self) -> bool {
        self.is_main
    }

    pub fn import_path(&self) -> ImportPath {
        if !self.prefix.is_empty() {
            ImportPath::new(
                std::iter::once(self.prefix.as_str())
                    .chain(std::iter::once(self.package.as_str()))
                    .chain(self.path.split('.'))
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("."),
            )
        } else {
            ImportPath::from(self.path.as_str())
        }
    }

    /// Check if this module path is part of the standard library.
    pub fn is_stdlib(&self) -> bool {
        self.prefix == "std"
    }

    /// Check if this module path is an external library.
    pub fn is_library(&self) -> bool {
        self.prefix == "lib"
    }

    /// Get the module path with underscore (_) separators instead of period (.)
    pub fn to_underscore(&self) -> String {
        std::iter::once(self.prefix.as_str())
            .chain(std::iter::once(self.package.as_str()))
            .chain(self.path.split('.'))
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("_")
    }
}

impl From<&PathBuf> for ModulePath {
    // Convert header path to module path
    // /lib/external/mylib.util.koi.h -> mylib.util
    fn from(p: &PathBuf) -> Self {
        let p = p.file_name().expect("expected filepath");
        let s = p.to_string_lossy().to_string();
        let s = s.trim_end_matches(".koi.h");
        let mut iter = s.split(".");
        let package = iter.next().expect("bad filepath");
        let path = iter.collect::<Vec<_>>().join(".");
        ModulePath::new("".into(), package.into(), path)
    }
}

impl From<ImportPath> for ModulePath {
    // Turns import path into module path
    // lib.mylib.core.util -> lib, mylib, core.util
    // server.router -> <empty>, <empty>, server.router
    fn from(impath: ImportPath) -> Self {
        if impath.is_library() || impath.is_stdlib() {
            let mut split = impath.path().split(".");
            let prefix = split.next().unwrap();
            let package = split
                .next()
                .expect("prefix without package name not allowed");
            let path = split.collect::<Vec<_>>().join(".");
            ModulePath::new(prefix.into(), package.into(), path)
        } else {
            ModulePath::new("".into(), "".into(), impath.path)
        }
    }
}

impl Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.import_path())
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct ImportPath {
    path: String,
}

impl Display for ImportPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl ImportPath {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Get only the module name (the last identifier of the path).
    pub fn name(&self) -> &str {
        &self
            .path
            .split(".")
            .last()
            .expect("called name on a non-import path")
    }

    /// Check if this module path is part of the standard library.
    pub fn is_stdlib(&self) -> bool {
        self.path.starts_with("std.")
    }

    /// Check if this module path is an external library.
    pub fn is_library(&self) -> bool {
        self.path.starts_with("lib.")
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl From<&str> for ImportPath {
    fn from(s: &str) -> Self {
        Self::new(s.into())
    }
}

impl From<String> for ImportPath {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&ModulePath> for ImportPath {
    fn from(modpath: &ModulePath) -> Self {
        modpath.import_path()
    }
}

impl From<&ImportNode> for ImportPath {
    fn from(import: &ImportNode) -> Self {
        import
            .names
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(".")
            .into()
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

        if module.modpath.is_main() {
            self.main_id = Some(id);
        }

        module
    }

    pub fn get(&self, id: ModuleId) -> Option<&Module> {
        assert!(id != invalid_mod_id(), "invalid mod id");
        self.modules.get(id)
    }

    /// Resolve a module path to a Module, referenced from the internal array.
    pub fn resolve(&self, impath: &ImportPath) -> Result<&Module, String> {
        self.cache
            .get(impath.path())
            .map_or(Err(format!("could not resolve module import")), |id| {
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
