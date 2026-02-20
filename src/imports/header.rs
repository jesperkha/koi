use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    ast::Pos,
    module::{
        CreateModule, ExternalModule, Module, ModuleKind, ModulePath, Symbol, SymbolKind,
        SymbolList, SymbolOrigin,
    },
    types::{PrimitiveType, TypeContext, TypeId, TypeKind},
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
    modpath: String,
    symbols: Vec<HeaderSymbol>,
    types: Vec<HeaderTypeKind>,
}

impl HeaderFile {
    /// Convert module to header file by extracting all exported symbols
    /// and types into parseable string representations.
    pub fn from_module(module: &Module, ctx: &TypeContext) -> HeaderFile {
        let mut mappings = HashMap::new();
        let mut types = Vec::new();

        let all_types_ids = module
            .exports()
            .values()
            .map(|symbol| ctx.get_all_references(symbol.ty))
            .flatten()
            .collect::<HashSet<_>>();

        for ty in all_types_ids {
            let kind = real_to_header(&ctx.lookup(ty).kind, ctx);
            let id = types.len();
            types.push(kind);
            mappings.insert(ty, id);
        }

        let mut header_symbols = Vec::new();
        for symbol in module.exports().values() {
            let header_symbol = HeaderSymbol {
                name: symbol.name.clone(),
                ty: *mappings
                    .get(&symbol.ty)
                    .expect("all types should be mapped"),
                kind: symbol.kind.clone(),
                no_mangle: symbol.no_mangle,
            };
            header_symbols.push(header_symbol);
        }

        HeaderFile {
            modpath: module.modpath.path().to_owned(),
            symbols: header_symbols,
            types,
        }
    }

    /// Parse this headers symbols content and create module.
    pub fn to_module(self, ctx: &mut TypeContext) -> Result<CreateModule, String> {
        let mut mappings = HashMap::new();
        for (header_id, header_kind) in self.types.iter().enumerate() {
            let real_id = header_to_real(header_kind, ctx);
            mappings.insert(header_id, real_id);
        }

        let modpath = ModulePath::new(self.modpath);

        let symbols = self
            .symbols
            .into_iter()
            .map(|s| Symbol {
                filename: "".into(),
                kind: s.kind,
                name: s.name,
                no_mangle: s.no_mangle,
                is_exported: true,   // Always true for imported symbols
                pos: Pos::default(), // Not used outside of type checking local modules anyways. TODO: remove pos from Symbol
                ty: *mappings.get(&s.ty).expect("mapping not found"),
                origin: SymbolOrigin::Extern(modpath.clone()), // Extern since we are linking
            })
            .collect::<Vec<_>>();

        Ok(CreateModule {
            modpath,
            kind: ModuleKind::External(ExternalModule {
                header_path: "".into(),
                archive_path: "".into(),
            }),
            symbols: SymbolList::new_from_list(symbols),
            deps: Vec::new(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct HeaderSymbol {
    name: String,
    ty: usize,
    kind: SymbolKind,
    no_mangle: bool,
}

/// Convert real type kind into header type kind.
fn real_to_header(kind: &TypeKind, ctx: &TypeContext) -> HeaderTypeKind {
    match kind {
        TypeKind::Primitive(p) => HeaderTypeKind::Primitive(p.into()),
        TypeKind::Array(id) => {
            HeaderTypeKind::Array(Box::new(real_to_header(&ctx.lookup(*id).kind, ctx)))
        }
        TypeKind::Pointer(id) => {
            HeaderTypeKind::Pointer(Box::new(real_to_header(&ctx.lookup(*id).kind, ctx)))
        }
        TypeKind::Alias(id) => {
            HeaderTypeKind::Alias(Box::new(real_to_header(&ctx.lookup(*id).kind, ctx)))
        }
        TypeKind::Unique(id) => {
            HeaderTypeKind::Unique(Box::new(real_to_header(&ctx.lookup(*id).kind, ctx)))
        }
        TypeKind::Function(func) => {
            let params = func
                .params
                .iter()
                .map(|id| real_to_header(&ctx.lookup(*id).kind, ctx))
                .collect();
            let ret = Box::new(real_to_header(&ctx.lookup(func.ret).kind, ctx));
            HeaderTypeKind::Function(params, ret)
        }
    }
}

/// Convert header type kind to real type kinds id.
fn header_to_real(kind: &HeaderTypeKind, ctx: &mut TypeContext) -> TypeId {
    let typekind = match kind {
        HeaderTypeKind::Primitive(p) => TypeKind::Primitive(p.into()),
        HeaderTypeKind::Array(inner) => TypeKind::Array(header_to_real(inner, ctx)),
        HeaderTypeKind::Pointer(inner) => TypeKind::Pointer(header_to_real(inner, ctx)),
        HeaderTypeKind::Alias(inner) => TypeKind::Alias(header_to_real(inner, ctx)),
        HeaderTypeKind::Unique(inner) => TypeKind::Unique(header_to_real(inner, ctx)),
        HeaderTypeKind::Function(params, ret) => {
            let param_ids = params.iter().map(|p| header_to_real(p, ctx)).collect();
            let ret_id = header_to_real(ret, ctx);
            TypeKind::Function(crate::types::FunctionType {
                params: param_ids,
                ret: ret_id,
            })
        }
    };
    ctx.get_or_intern(typekind)
}

#[derive(Debug, Serialize, Deserialize)]
enum HeaderTypeKind {
    Primitive(HeaderPrimitiveType),
    Array(Box<HeaderTypeKind>),
    Pointer(Box<HeaderTypeKind>),
    Alias(Box<HeaderTypeKind>),
    Unique(Box<HeaderTypeKind>),
    Function(Vec<HeaderTypeKind>, Box<HeaderTypeKind>),
}

#[derive(Debug, Serialize, Deserialize)]
enum HeaderPrimitiveType {
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
