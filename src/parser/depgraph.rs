use std::collections::HashMap;

use petgraph::{
    algo::{has_path_connecting, toposort},
    prelude::DiGraphMap,
};
use tracing::info;

use crate::ast::FileSet;

fn is_stdlib(id: &str) -> bool {
    vec![].contains(&id)
}

/// Sort list of FileSets based on their imports by creating a dependency graph.
/// The first element in the returned list is the least depended on package
/// and must be type checked first.
pub fn sort_by_dependency_graph(sets: Vec<FileSet>) -> Result<Vec<FileSet>, String> {
    assert!(sets.len() > 0, "empty set list");

    let mut index = HashMap::new();
    let mut dag: DiGraphMap<usize, ()> = DiGraphMap::new();

    for fs in &sets {
        let id = index.len();
        index.insert(fs.module_path.clone(), id);
        dag.add_node(id);
    }

    for fs in &sets {
        for import in &fs.imports {
            if is_stdlib(&import.module_path) {
                continue;
            }

            let Some(a) = index.get(&import.module_path) else {
                continue; // Handled in import resolution
            };

            let b = *index.get(&fs.module_path).expect("missing import {}");

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
        let id = index[&fs.module_path];
        id_to_fileset.insert(id, fs);
    }

    let sorted_sets: Vec<FileSet> = sorted_ids
        .into_iter()
        .map(|id| id_to_fileset.remove(&id).unwrap())
        .collect();

    info!(
        "final check order: {}",
        sorted_sets
            .iter()
            .map(|s| s.module_path.clone())
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(sorted_sets)
}
