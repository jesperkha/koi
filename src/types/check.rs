use crate::{
    ast::{File, FileSet},
    config::Config,
    error::{ErrorSet, Res},
    types::{Checker, Package, TypeContext},
};
use tracing::info;

// TODO: Complete imports
// 1. Scan each file in package and collect all exported items into Exports
// 2. Create a map of all exports in the project, including std and external imports
// 3. Type check each package using this import map
// 4. Checker now only accepts a list of Decl, typecontext

/*
    exports, ctx = collect_exports(file)
    pkg = check(export, ctx, file)
*/

pub fn check_fileset(fs: FileSet, config: &Config) -> Res<Package> {
    let mut ctx = TypeContext::new();
    let mut errs = ErrorSet::new();

    info!("checking {} files", fs.files.len());

    // TODO: remove this check and handle empty packages properly
    assert!(fs.files.len() > 0, "no files to type check");

    for file in &fs.files {
        let checker = Checker::new(&file, &mut ctx, config);
        errs.join(checker.check());
    }

    if errs.len() > 0 {
        info!("fail, finished all with {} errors", errs.len());
        return Err(errs);
    }

    // TODO: assert all pkg names equal

    info!("success, all files");
    Ok(Package::new(
        fs.package_id.0.clone(),
        // TODO: filepath in packages, copy from file
        "".to_string(),
        fs.files,
        ctx,
    ))
}
