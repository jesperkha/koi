use std::collections::HashSet;

use crate::ast::{File, FileSet, PackageID};

pub fn new_fileset(files: Vec<File>) -> FileSet {
    let package_id = if files.len() == 0 {
        PackageID(files[0].package_name.clone())
    } else {
        PackageID(String::from(""))
    };

    let mut imports = HashSet::new();

    for file in &files {
        for imp in &file.ast.imports {
            imports.insert(PackageID(
                imp.names
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join("."),
            ));
        }
    }

    FileSet {
        package_id,
        imports,
        files,
    }
}
