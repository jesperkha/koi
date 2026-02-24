use std::collections::HashSet;

use crate::{
    ast::{FileSet, Node},
    error::{Diagnostics, Report, Res},
    module::ImportPath,
};

/// Checks all imports for external modules (std. or lib.) and reports errors
/// if the library import is not part of the known list of external libraries.
pub fn validate_imports(fs: &FileSet, libraries: HashSet<ImportPath>) -> Res<()> {
    let mut diag = Diagnostics::new();

    for file in &fs.files {
        for import in &file.ast.imports {
            let impath = ImportPath::from(import);

            // If stdlib/external and it exists in /lib, skip
            if !(impath.is_stdlib() || impath.is_library()) || libraries.contains(&impath) {
                continue;
            }

            diag.add(Report::code_error(
                "could not resolve library import", // TODO: better error message?
                import.pos(),
                import.end(),
            ));
        }
    }

    diag.err_or(())
}
