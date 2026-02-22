use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    ast::Pos,
    module::{
        CreateModule, Module, ModuleKind, ModulePath, Symbol, SymbolKind, SymbolList, SymbolOrigin,
    },
    types::{PrimitiveType, TypeContext, TypeId, TypeKind},
};

/// Create a header file from a module's exported symbols and types.
pub fn create_header_file(module: &Module, ctx: &TypeContext) -> Result<Vec<u8>, String> {
    let header = HeaderFile::from_module(module, ctx);
    postcard::to_stdvec(&header).map_err(|e| e.to_string())
}

/// Parse header file and intern types in context. Return the created module.
pub fn read_header_file(
    modpath: ModulePath,
    bytes: &[u8],
    ctx: &mut TypeContext,
) -> Result<CreateModule, String> {
    let header: HeaderFile = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    header.to_module(modpath, ctx)
}

#[derive(Debug, Serialize, Deserialize)]
struct HeaderFile {
    symbols: Vec<HeaderSymbol>,
    types: Vec<HeaderTypeKind>,
}

impl HeaderFile {
    /// Convert module to header file by extracting all exported symbols
    /// and types into parseable string representations.
    pub fn from_module(module: &Module, ctx: &TypeContext) -> HeaderFile {
        let mut mappings = HashMap::new();
        let mut types = Vec::new();

        // Get all type ids used in this module
        let all_types_ids = module
            .exports()
            .values()
            .map(|symbol| ctx.get_all_references(symbol.ty))
            .flatten()
            .collect::<HashSet<_>>();

        // Create a map from TypeId to HeaderTypeKind to store in the header file
        for ty in all_types_ids {
            let kind = real_to_header(&ctx.lookup(ty).kind, ctx);
            let id = types.len();
            types.push(kind);
            mappings.insert(ty, id);
        }

        // Convert all symbols to HeaderSymbol
        let symbols = module
            .exports()
            .iter()
            .map(|(_, symbol)| HeaderSymbol {
                name: symbol.name.clone(),
                ty: *mappings
                    .get(&symbol.ty)
                    .expect("all types should be mapped"),
                kind: symbol.kind.clone(),
                no_mangle: symbol.no_mangle,
            })
            .collect();

        HeaderFile { symbols, types }
    }

    /// Parse this headers symbols content and create module.
    pub fn to_module(
        self,
        modpath: ModulePath,
        ctx: &mut TypeContext,
    ) -> Result<CreateModule, String> {
        // Create map of header id to real id (HeaderTypeId -> TypeId)
        let mappings = self
            .types
            .iter()
            .enumerate()
            .map(|(hid, hkind)| (hid, header_to_real(hkind, ctx)))
            .collect::<HashMap<_, _>>();

        // Convert header symbols to Symbols
        let symbols = self
            .symbols
            .into_iter()
            .map(|s| Symbol {
                filename: "".into(), // TODO: resolve filename for header module
                kind: s.kind,
                name: s.name,
                no_mangle: s.no_mangle,
                is_exported: true,   // Always true for imported symbols
                pos: Pos::default(), // Not used outside of type checking local modules anyways. TODO: remove pos from Symbol
                ty: *mappings.get(&s.ty).expect("mapping not found"),
                origin: SymbolOrigin::Module(modpath.clone()), // Module since we use mangling
            })
            .collect::<Vec<_>>();

        Ok(CreateModule {
            modpath,
            kind: ModuleKind::External,
            symbols: SymbolList::from(symbols),
            deps: Vec::new(),
        })
    }
}

/// HeaderSymbol is the header representation of a Symbol from a module.
/// It is lossless, meaning all information can be recovered from it.
/// Missing fields from the original Symbol type are not included as they
/// are the same for all HeaderSymbol.
#[derive(Debug, Serialize, Deserialize)]
struct HeaderSymbol {
    name: String,
    ty: usize,
    kind: SymbolKind,
    no_mangle: bool,
}

/// HeaderTypeKind is the header representation of a TypeKind from the type context.
#[derive(Debug, Serialize, Deserialize)]
enum HeaderTypeKind {
    Primitive(HeaderPrimitiveType),
    Array(Box<HeaderTypeKind>),
    Pointer(Box<HeaderTypeKind>),
    Alias(Box<HeaderTypeKind>),
    Unique(Box<HeaderTypeKind>),
    Function(Vec<HeaderTypeKind>, Box<HeaderTypeKind>),
}

/// Convert real type kind into header type kind.
fn real_to_header(kind: &TypeKind, ctx: &TypeContext) -> HeaderTypeKind {
    match kind {
        TypeKind::Primitive(p) => HeaderTypeKind::Primitive(p.into()),
        TypeKind::Array(id) => HeaderTypeKind::Array(boxed_kind(*id, ctx)),
        TypeKind::Pointer(id) => HeaderTypeKind::Pointer(boxed_kind(*id, ctx)),
        TypeKind::Alias(id) => HeaderTypeKind::Alias(boxed_kind(*id, ctx)),
        TypeKind::Unique(id) => HeaderTypeKind::Unique(boxed_kind(*id, ctx)),
        TypeKind::Function(func) => {
            let params = func
                .params
                .iter()
                .map(|id| real_to_header(&ctx.lookup(*id).kind, ctx))
                .collect();
            let ret = boxed_kind(func.ret, ctx);
            HeaderTypeKind::Function(params, ret)
        }
    }
}

fn boxed_kind(ty: TypeId, ctx: &TypeContext) -> Box<HeaderTypeKind> {
    Box::new(real_to_header(&ctx.lookup(ty).kind, ctx))
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
            PrimitiveType::Void => Self::Void,
            PrimitiveType::I8 => Self::I8,
            PrimitiveType::I16 => Self::I16,
            PrimitiveType::I32 => Self::I32,
            PrimitiveType::I64 => Self::I64,
            PrimitiveType::U8 => Self::U8,
            PrimitiveType::U16 => Self::U16,
            PrimitiveType::U32 => Self::U32,
            PrimitiveType::U64 => Self::U64,
            PrimitiveType::F32 => Self::F32,
            PrimitiveType::F64 => Self::F64,
            PrimitiveType::Bool => Self::Bool,
            PrimitiveType::Byte => Self::Byte,
            PrimitiveType::String => Self::String,
        }
    }
}

impl From<&HeaderPrimitiveType> for PrimitiveType {
    fn from(p: &HeaderPrimitiveType) -> Self {
        match p {
            HeaderPrimitiveType::Void => Self::Void,
            HeaderPrimitiveType::I8 => Self::I8,
            HeaderPrimitiveType::I16 => Self::I16,
            HeaderPrimitiveType::I32 => Self::I32,
            HeaderPrimitiveType::I64 => Self::I64,
            HeaderPrimitiveType::U8 => Self::U8,
            HeaderPrimitiveType::U16 => Self::U16,
            HeaderPrimitiveType::U32 => Self::U32,
            HeaderPrimitiveType::U64 => Self::U64,
            HeaderPrimitiveType::F32 => Self::F32,
            HeaderPrimitiveType::F64 => Self::F64,
            HeaderPrimitiveType::Bool => Self::Bool,
            HeaderPrimitiveType::Byte => Self::Byte,
            HeaderPrimitiveType::String => Self::String,
        }
    }
}
