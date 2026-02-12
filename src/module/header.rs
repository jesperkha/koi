use serde::{Deserialize, Serialize};

use crate::{
    ast::{Source, SourceMap},
    config::Config,
    module::{CreateModule, Module, ModuleGraph},
    parser::parse_source_map,
    typecheck::check_fileset,
    types::TypeContext,
};

#[derive(Debug, Serialize, Deserialize)]
struct HeaderFile {
    filename: String,
    modpath: String,
    symbols: String,
}

impl HeaderFile {
    /// Convert module to header file by extracting all exported symbols
    /// and types into parseable string representations.
    pub fn from_module(module: &Module, ctx: &TypeContext) -> HeaderFile {
        HeaderFile {
            filename: module.name().to_owned(),
            modpath: module.modpath.path().to_owned(),
            symbols: module
                .exports()
                .values()
                .map(|s| s.to_header_format(ctx))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }

    /// Parse this headers symbols content and create module.
    pub fn to_module(
        self,
        mg: &ModuleGraph,
        ctx: &mut TypeContext,
    ) -> Result<CreateModule, String> {
        let config = Config::default();
        let source = Source::new_str(self.filename, self.symbols);
        let map = SourceMap::one(source);
        let modpath = self.modpath.into();
        let fs = parse_source_map(modpath, &map, &config).map_err(|err| err.render(&map))?;
        check_fileset(fs, mg, ctx, &config).map_err(|err| err.render(&map))
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
    let create_mod = header.to_module(mg, ctx)?;
    let module = mg.add(create_mod);
    Ok(module)
}
