use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    module::{CreateModule, Module, SymbolKind},
    types::{TypeContext, TypeKind},
};

/// Return header file contents for a given module. All exported symbols of
/// the given module are included and neatly formatted with docs.
pub fn create_header_file(module: &Module, ctx: &TypeContext) -> Result<Vec<u8>, String> {
    let header = HeaderFile::from_module(module, ctx);
    postcard::to_stdvec(&header).map_err(|e| e.to_string())
}

/// Parse and type check a header file from source string, adding it to the module graph.
pub fn read_header_file(bytes: &[u8], ctx: &mut TypeContext) -> Result<CreateModule, String> {
    let header: HeaderFile = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    header.to_module(ctx)
}

#[derive(Debug, Serialize, Deserialize)]
struct HeaderFile {
    filename: String,
    modpath: String,
    symbols: Vec<HeaderSymbol>,
    typemap: HashMap<HeaderTypeId, TypeKind>,
}

impl HeaderFile {
    /// Convert module to header file by extracting all exported symbols
    /// and types into parseable string representations.
    pub fn from_module(module: &Module, ctx: &TypeContext) -> HeaderFile {
        todo!()
    }

    /// Parse this headers symbols content and create module.
    pub fn to_module(self, ctx: &mut TypeContext) -> Result<CreateModule, String> {
        todo!()
    }
}

type HeaderTypeId = usize;

// TODO: (current) header files
// 1. Create new smaller type context from symbol types
// 2. When reading the header back in, put the type kinds
//    into TypeContext and map the updated TypeId to the Symbol.

#[derive(Debug, Serialize, Deserialize)]
struct HeaderSymbol {
    name: String,
    ty: HeaderTypeId,
    kind: SymbolKind,
    no_mangle: bool,
}
