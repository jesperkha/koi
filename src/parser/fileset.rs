use std::collections::HashSet;

use crate::ast::{File, FileSet, PackageID};

pub fn new_fileset(files: Vec<File>) -> Result<FileSet, String> {
    if files.len() == 0 {
        return Err(format!("no input files"));
    }

    let package_id = PackageID(files[0].package_name.clone());
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

    Ok(FileSet {
        package_id,
        imports,
        files,
    })
}
