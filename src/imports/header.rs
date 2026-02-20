use std::collections::HashMap;

use postcard::fixint::le;
use serde::{Deserialize, Serialize};

use crate::{
    ast::{Source, SourceMap},
    config::Config,
    module::{
        CreateModule, ExternalModule, FuncSymbol, Module, ModuleGraph, ModuleKind, ModulePath,
        Symbol, SymbolKind,
    },
    parser::parse_source_map,
    typecheck::check_fileset,
    types::{PrimitiveType, TypeContext, TypeId, TypeKind},
};

/// Return header file contents for a given module. All exported symbols of
/// the given module are included and neatly formatted with docs.
pub fn create_header_file(module: &Module, ctx: &TypeContext) -> Result<Vec<u8>, String> {
    let header = HeaderFile::from_module(module, ctx);
    postcard::to_stdvec(&header).map_err(|e| e.to_string())
}

/// Parse and type check a header file from source string, adding it to the module graph.
pub fn read_header_file(
    bytes: &[u8],
    mg: &ModuleGraph,
    ctx: &mut TypeContext,
) -> Result<CreateModule, String> {
    let header: HeaderFile = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    header.to_module(mg, ctx)
}

#[derive(Debug, Serialize, Deserialize)]
struct HeaderFile {
    filename: String,
    modpath: String,
    symbols: Vec<HeaderSymbol>,
    typemap: HeaderTypeMap,
}

impl HeaderFile {
    /// Convert module to header file by extracting all exported symbols
    /// and types into parseable string representations.
    pub fn from_module(module: &Module, ctx: &TypeContext) -> HeaderFile {
        let (symbols, typemap) = convert_to_header_symbols(module, ctx);
        HeaderFile {
            filename: module.name().to_owned(),
            modpath: module.modpath.path().to_owned(),
            symbols,
            typemap,
        }
    }

    /// Parse this headers symbols content and create module.
    pub fn to_module(
        self,
        mg: &ModuleGraph,
        ctx: &mut TypeContext,
    ) -> Result<CreateModule, String> {
        let create_mod = CreateModule {
            modpath: ModulePath::new(self.modpath),
            kind: ModuleKind::External(ExternalModule {
                header_path: self.filename.clone(),
                archive_path: self.filename.clone(),
            }),
            deps: Vec::new(),
            symbols: todo!(),
        };

        Ok(create_mod)
    }
}

// TODO: instead of using text header, make a serializable Symbol type and read in that.

fn convert_to_header_symbols(
    module: &Module,
    ctx: &TypeContext,
) -> (Vec<HeaderSymbol>, HeaderTypeMap) {
    let mut map = HeaderTypeMap::new();
    let mut header_symbols = Vec::new();

    for s in module.exports().values() {
        let typeid = map.get(s.ty, ctx);
        header_symbols.push(HeaderSymbol {
            name: s.name.clone(),
            ty: typeid,
            kind: (&s.kind).into(),
            no_mangle: s.no_mangle,
        });
    }

    (header_symbols, map)
}

#[derive(Debug, Serialize, Deserialize)]
struct HeaderSymbol {
    name: String,
    ty: usize,
    kind: HeaderSymbolKind,
    no_mangle: bool,
}

#[derive(Debug, Serialize, Deserialize)]
enum HeaderSymbolKind {
    Function { is_inline: bool, is_naked: bool },
}

impl From<&SymbolKind> for HeaderSymbolKind {
    fn from(f: &SymbolKind) -> Self {
        match f {
            SymbolKind::Function(f) => HeaderSymbolKind::Function {
                is_inline: f.is_inline,
                is_naked: f.is_naked,
            },
        }
    }
}

impl From<&HeaderSymbolKind> for SymbolKind {
    fn from(k: &HeaderSymbolKind) -> Self {
        match k {
            HeaderSymbolKind::Function {
                is_inline,
                is_naked,
            } => SymbolKind::Function(FuncSymbol {
                is_inline: *is_inline,
                is_naked: *is_naked,
                docs: Vec::new(),
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum HeaderTypeKind {
    Primitive(HeaderPrimitiveType),
    Alias(usize),
    Unique(usize),
    Pointer(usize),
    Array(usize),
    Function(Vec<usize>, usize),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HeaderPrimitiveType {
    Void,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Byte,
    String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HeaderTypeMap {
    types: Vec<HeaderTypeKind>,
    mappings: HashMap<TypeId, usize>,
}

impl HeaderTypeMap {
    fn new() -> Self {
        Self {
            types: Vec::new(),
            mappings: HashMap::new(),
        }
    }

    fn get(&mut self, ty: TypeId, ctx: &TypeContext) -> usize {
        let Some(idx) = self.mappings.get(&ty) else {
            let kind = &ctx.lookup(ty).kind;
            let header_kind = self.to_header_kind(kind, ctx);
            let idx = self.types.len();
            self.types.push(header_kind);
            self.mappings.insert(ty, idx);
            return idx;
        };
        *idx
    }

    fn to_header_kind(&mut self, kind: &TypeKind, ctx: &TypeContext) -> HeaderTypeKind {
        match kind {
            TypeKind::Primitive(p) => HeaderTypeKind::Primitive(p.into()),
            TypeKind::Function(f) => HeaderTypeKind::Function(
                f.params.iter().map(|p| self.get(*p, ctx)).collect(),
                self.get(f.ret, ctx),
            ),
            TypeKind::Array(ty) => HeaderTypeKind::Array(self.get(*ty, ctx)),
            TypeKind::Pointer(ty) => HeaderTypeKind::Pointer(self.get(*ty, ctx)),
            TypeKind::Alias(ty) => HeaderTypeKind::Alias(self.get(*ty, ctx)),
            TypeKind::Unique(ty) => HeaderTypeKind::Unique(self.get(*ty, ctx)),
        }
    }

    fn to_type_kind(&self, kind: &HeaderTypeKind, ctx: &TypeContext) -> TypeKind {
        todo!()
    }
}

impl From<&PrimitiveType> for HeaderPrimitiveType {
    fn from(p: &PrimitiveType) -> Self {
        match p {
            PrimitiveType::Void => HeaderPrimitiveType::Void,
            PrimitiveType::I8 => HeaderPrimitiveType::I8,
            PrimitiveType::I16 => HeaderPrimitiveType::I16,
            PrimitiveType::I32 => HeaderPrimitiveType::I32,
            PrimitiveType::I64 => HeaderPrimitiveType::I64,
            PrimitiveType::U8 => HeaderPrimitiveType::U8,
            PrimitiveType::U16 => HeaderPrimitiveType::U16,
            PrimitiveType::U32 => HeaderPrimitiveType::U32,
            PrimitiveType::U64 => HeaderPrimitiveType::U64,
            PrimitiveType::F32 => HeaderPrimitiveType::F32,
            PrimitiveType::F64 => HeaderPrimitiveType::F64,
            PrimitiveType::Bool => HeaderPrimitiveType::Bool,
            PrimitiveType::Byte => HeaderPrimitiveType::Byte,
            PrimitiveType::String => HeaderPrimitiveType::String,
        }
    }
}

impl From<&HeaderPrimitiveType> for PrimitiveType {
    fn from(p: &HeaderPrimitiveType) -> Self {
        match p {
            HeaderPrimitiveType::Void => PrimitiveType::Void,
            HeaderPrimitiveType::I8 => PrimitiveType::I8,
            HeaderPrimitiveType::I16 => PrimitiveType::I16,
            HeaderPrimitiveType::I32 => PrimitiveType::I32,
            HeaderPrimitiveType::I64 => PrimitiveType::I64,
            HeaderPrimitiveType::U8 => PrimitiveType::U8,
            HeaderPrimitiveType::U16 => PrimitiveType::U16,
            HeaderPrimitiveType::U32 => PrimitiveType::U32,
            HeaderPrimitiveType::U64 => PrimitiveType::U64,
            HeaderPrimitiveType::F32 => PrimitiveType::F32,
            HeaderPrimitiveType::F64 => PrimitiveType::F64,
            HeaderPrimitiveType::Bool => PrimitiveType::Bool,
            HeaderPrimitiveType::Byte => PrimitiveType::Byte,
            HeaderPrimitiveType::String => PrimitiveType::String,
        }
    }
}
