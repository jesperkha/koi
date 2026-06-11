use std::collections::HashSet;

use tracing::debug;

use koi_ast::{FileSet, ImportPath, Node};
use koi_common::error::{Diagnostics, Report, Res};

pub fn validate_imports(fs: &FileSet, libraries: HashSet<ImportPath>) -> Res<()> {
    let mut diag = Diagnostics::new();

    for file in &fs.files {
        for import in &file.ast.imports {
            let impath = ImportPath::from(import);

            if !(impath.is_stdlib() || impath.is_library()) || libraries.contains(&impath) {
                continue;
            }

            debug!("ImportPath={:?}", impath);
            debug!(
                "Available=[{}]",
                libraries
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            diag.add(Report::code_error(
                "could not resolve library import",
                import.pos(),
                import.end(),
            ));
        }
    }

    diag.err_or(())
}
