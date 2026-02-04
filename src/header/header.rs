use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    error::ErrorSet,
    module::{Module, ModuleGraph, ModulePath},
    parser::scan_and_parse,
    typecheck::check_header_file,
    types::TypeContext,
};

// TODO: test headers

#[derive(Debug, Serialize, Deserialize)]
struct HeaderFile {
    name: String,
    symbols: String,
}

impl HeaderFile {
    /// Convert module to header file by extracting all exported symbols
    /// and types into parseable string representations.
    pub fn from_module(module: &Module, ctx: &TypeContext) -> HeaderFile {
        HeaderFile {
            name: module.name().to_owned(),
            symbols: module
                .exports()
                .values()
                .map(|s| s.to_header_format(ctx))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    /// Parse this headers symbols content and create module.
    pub fn to_module<'a>(
        &self,
        mg: &'a mut ModuleGraph,
        ctx: &mut TypeContext,
    ) -> Result<&'a Module, ErrorSet> {
        let config = Config::test();
        let file = scan_and_parse(&self.symbols, &config)?;
        let modpath = ModulePath::new(self.name.clone());
        let createmod = check_header_file(&modpath, file, ctx, &config)?;
        Ok(mg.add(createmod))
    }
}

/// Return header file contents for a given module. All exported symbols of
/// the given module are included and neatly formatted with docs.
pub fn create_header_file(module: &Module, ctx: &TypeContext) -> Result<Vec<u8>, String> {
    let header = HeaderFile::from_module(module, ctx);
    postcard::to_stdvec(&header).map_err(|e| e.to_string())
}

/// Parse and type check a header file from source string, adding it to the module graph.
pub fn read_header_file<'a>(
    bytes: &[u8],
    mg: &'a mut ModuleGraph,
    ctx: &mut TypeContext,
) -> Result<&'a Module, String> {
    let header: HeaderFile = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    header.to_module(mg, ctx).map_err(|e| e.to_string())
}
