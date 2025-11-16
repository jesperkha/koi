use std::collections::HashMap;

use petgraph::{
    algo::{has_path_connecting, toposort},
    prelude::DiGraphMap,
};

use crate::ast::{FileSet, PackageID};

fn is_stdlib(id: &PackageID) -> bool {
    vec!["io", "os", "http", "str"].contains(&id.0.as_str())
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
            if is_stdlib(import) {
                continue;
            }

            let a = *index
                .get(&import.0)
                .expect(format!("missing import {}", &import.0).as_str());

            let b = *index.get(&fs.package_id.0).expect("missing import {}");

            if has_path_connecting(&dag, b, a, None) {
                return Err(format!("import cycle detected"));
            }

            dag.add_edge(a, b, ());
        }
    }

    let sorted_ids =
        toposort(&dag, None).map_err(|_| "cycle detected in dependencies".to_string())?;

    let mut id_to_fileset = HashMap::new();
    for fs in sets {
        let id = index[&fs.package_id.0];
        id_to_fileset.insert(id, fs);
    }

    let sorted_sets = sorted_ids
        .into_iter()
        .map(|id| id_to_fileset.remove(&id).unwrap())
        .collect();

    Ok(sorted_sets)
}
