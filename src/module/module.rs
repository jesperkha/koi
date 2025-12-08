use std::collections::HashMap;

use crate::{module::Exports, types::TypedAst};

pub enum ModuleKind {
    Stdlib,
    User,
    ThirdParty,
}

pub type ModuleId = usize;

pub fn invalid_mod_id() -> ModuleId {
    return usize::MAX;
}

pub struct CreateModule {
    pub name: String,
    pub path: String,
    pub ast: TypedAst,
    pub exports: Exports,
    pub kind: ModuleKind,
}

/// A Module is a self-contained compilation unit. It contains the combined
/// typed AST of all files in the module and all exported symbols.
pub struct Module {
    pub id: ModuleId,
    /// The id of this modules parent. 0 means this is root.
    pub parent: ModuleId,
    /// The module name (name of directory, not path).
    pub name: String,
    /// The relative path from src to this module.
    pub path: String,
    /// The fully typed AST generated from files in this module.
    pub ast: TypedAst,
    /// All symbols exported by this module.
    pub exports: Exports,
    /// What type of module this is.
    pub kind: ModuleKind,
}

pub struct ModuleGraph {
    pub modules: Vec<Module>,
    pub children: HashMap<ModuleId, Vec<ModuleId>>,
    pub roots: Vec<ModuleId>,
}

impl ModuleGraph {
    pub fn new() -> Self {
        ModuleGraph {
            modules: Vec::new(),
            children: HashMap::new(),
            roots: Vec::new(),
        }
    }

    /// Create a new module and add it to the graph. If the parent is not
    /// invalid then this module will be added as its child, otherwise it
    /// will be registered as a root module.
    pub fn add(&mut self, m: CreateModule, parent: ModuleId) -> &Module {
        let id = self.modules.len();

        if parent != invalid_mod_id() {
            // If this has a parent we add it as a child to the parent childrens list
            if let Some(v) = self.children.get_mut(&parent) {
                v.push(id);
            } else {
                self.children.insert(parent, vec![id]);
            }
        } else {
            // Otherwise this is a root module
            self.roots.push(id);
        }

        self.modules.push(Module {
            id,
            parent,
            name: m.name,
            path: m.path,
            ast: m.ast,
            exports: m.exports,
            kind: m.kind,
        });
        &self.modules[id]
    }

    pub fn get(&self, id: ModuleId) -> Option<&Module> {
        assert!(id != invalid_mod_id(), "invalid mod id");
        self.modules.get(id)
    }

    /// Resolve a module path to a Module, referenced from the internal array.
    pub fn resolve(&self, names: &[String]) -> Result<&Module, String> {
        let mut child_ids = &self.roots;

        for (i, name) in names.iter().enumerate() {
            for id in child_ids {
                let module = self.get(*id).expect("implementation error");

                if &module.name == name {
                    // If this is the last name return the module
                    if i == names.len() - 1 {
                        return Ok(module);
                    }

                    // Otherwise get its children and continue
                    if let Some(new_ids) = self.children_ids(*id) {
                        child_ids = new_ids;
                    } else {
                        return Err(format!("module '{}' has no submodules", module.name));
                    }
                }
            }
        }

        Err(format!("could not resolve module path"))
    }

    fn children_ids(&self, id: ModuleId) -> Option<&Vec<ModuleId>> {
        self.children.get(&id)
    }

    fn children(&self, id: ModuleId) -> Option<Vec<&Module>> {
        assert!(id != invalid_mod_id(), "invalid mod id");
        if let Some(ids) = self.children.get(&id) {
            Some(ids.iter().map(|id| &self.modules[*id]).collect())
        } else {
            None
        }
    }

    fn ast(&self, id: ModuleId) -> &TypedAst {
        assert!(id != invalid_mod_id(), "invalid mod id");
        &self.modules[id].ast
    }
}
