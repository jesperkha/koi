use crate::{
    error::Res,
    module::{Module, ModuleGraph},
    types::TypeContext,
};

/// Return header file contents for a given module. All exported symbols of
/// the given module are included and neatly formatted with docs.
pub fn create_header_file(_module: &Module, _ctx: &TypeContext) -> Result<String, String> {
    todo!()
}

/// Parse and type check a header file from source string, adding it to the module graph.
pub fn read_header_file<'a>(_mg: &'a mut ModuleGraph, _ctx: &mut TypeContext) -> Res<&'a Module> {
    todo!()
}
