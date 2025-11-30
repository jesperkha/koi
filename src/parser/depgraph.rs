use std::collections::HashMap;

use petgraph::{
    algo::{has_path_connecting, toposort},
    prelude::DiGraphMap,
};
use tracing::info;

use crate::ast::{FileSet, PackageID};

fn is_stdlib(id: &PackageID) -> bool {
    vec![].contains(&id.0.as_str())
}

pub fn sort_by_dependency_graph(sets: Vec<FileSet>) -> Result<Vec<FileSet>, String> {
    let mut index = HashMap::new();
    let mut dag: DiGraphMap<usize, ()> = DiGraphMap::new();

    for fs in &sets {
        let id = index.len();
        index.insert(fs.package_id.0.clone(), id);
        dag.add_node(id);
    }

    for fs in &sets {
        for import in &fs.imports {
            if is_stdlib(&import.name) {
                continue;
            }

            let Some(a) = index.get(&import.name.0) else {
                continue; // Handled in import resolution
            };

            let b = *index.get(&fs.package_id.0).expect("missing import {}");

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
        let id = index[&fs.package_id.0];
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
            .map(|s| s.package_id.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(sorted_sets)
}
