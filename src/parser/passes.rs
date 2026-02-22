use std::collections::HashSet;

use crate::{
    ast::{FileSet, Node},
    error::{Diagnostics, Report, Res},
    module::ModulePath,
};

/// Checks all imports for external modules (std. or lib.) and reports errors
/// if the library import is not part of the known list of external libraries.
pub fn validate_imports(fs: &FileSet, libraries: HashSet<&ModulePath>) -> Res<()> {
    let mut diag = Diagnostics::new();

    for file in &fs.files {
        for import in &file.ast.imports {
            let modpath = ModulePath::from(import);

            // If stdlib/external and it exists in /lib, skip
            if !(modpath.is_stdlib() || modpath.is_library()) || libraries.contains(&modpath) {
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
