use std::collections::HashMap;

use petgraph::{
    algo::{has_path_connecting, toposort},
    prelude::DiGraphMap,
};
use tracing::info;

use crate::{ast::FileSet, module::ImportPath};

pub struct SortResult {
    pub sets: Vec<FileSet>,
    pub external_imports: Vec<ImportPath>,
}

/// Sort list of FileSets based on their imports by creating a dependency graph.
/// The first element in the returned ordered list is the least depended on module
/// and must be type checked first.
pub fn sort_by_dependency_graph(sets: Vec<FileSet>) -> Result<SortResult, String> {
    if sets.len() == 0 {
        return Ok(SortResult {
            sets: Vec::new(),
            external_imports: Vec::new(),
        });
    }

    let mut index = HashMap::new();
    let mut dag: DiGraphMap<usize, ()> = DiGraphMap::new();

    for fs in &sets {
        let id = index.len();
        index.insert(fs.modpath.path().to_owned(), id);
        dag.add_node(id);
    }

    let mut external_imports = Vec::new();

    for fs in &sets {
        for import in &fs.imports {
            let import_path = import.impath.path();
            let fs_path = fs.modpath.path();

            if import_path == fs_path {
                return Err(format!("import cycle detected"));
            }

            // Stdlib and external imports are resolved elsewhere and are
            // guaranteed to be present when type checking the source code.
            if import.impath.is_library() || import.impath.is_stdlib() {
                external_imports.push(import_path.into());
                continue;
            }

            let Some(a) = index.get(import_path) else {
                continue; // Handled in import resolution
            };

            let b = *index.get(fs_path).expect("missing import {}");

            if has_path_connecting(&dag, b, *a, None) {
                return Err(format!("import cycle detected"));
            }

            dag.add_edge(*a, b, ());
        }
    }

    let sorted_ids =
        toposort(&dag, None).map_err(|_| "cycle detected in dependencies".to_string())?;

    let mut id_to_fileset = HashMap::new();
    for fs in sets {
        let id = index[fs.modpath.path()];
        id_to_fileset.insert(id, fs);
    }

    let ordered: Vec<FileSet> = sorted_ids
        .into_iter()
        .map(|id| id_to_fileset.remove(&id).unwrap())
        .collect();

    info!(
        "Final ordered module dependency list: {}",
        ordered
            .iter()
            .map(|s| s.modpath.path().to_owned())
            .collect::<Vec<_>>()
            .join(" -> ")
    );

    Ok(SortResult {
        sets: ordered,
        external_imports,
    })
}
