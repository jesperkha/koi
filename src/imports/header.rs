use std::{
    collections::{HashMap, HashSet},
    fs::read,
};

use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    context::{Context, CreateModule, CreateSymbol},
    module::{
        ImportPath, Module, ModuleId, ModuleKind, ModulePath, ModuleSymbol, ModuleSymbolKind,
        SymbolKind, SymbolOrigin,
    },
    types::{PrimitiveType, TypeId, TypeKind},
};

/// Create a header file from a module's exported symbols and types.
pub fn create_header_file(ctx: &Context, id: ModuleId) -> Result<Vec<u8>, String> {
    let module = ctx.modules.get(id);
    let header = HeaderFile::from_module(ctx, module);
    postcard::to_stdvec(&header).map_err(|e| e.to_string())
}

/// Parse header file and intern types in context. Return the created module.
pub fn read_header_file(
    ctx: &mut Context,
    modpath: ModulePath,
    bytes: &[u8],
) -> Result<CreateModule, String> {
    let header: HeaderFile = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    header.to_module(ctx, modpath)
}

pub fn dump_header_symbols(filepath: &str) -> Result<String, String> {
    let modpath = ModulePath::from(ImportPath::from("header"));
    let bytes = read(filepath).map_err(|e| format!("failed to read header file: {}", e))?;
    let mut ctx = Context::new(Config::default());
    let module = read_header_file(&mut ctx, modpath, &bytes)?;
    Ok(module.symbols.dump(&ctx, filepath))
}

#[derive(Debug, Serialize, Deserialize)]
struct HeaderFile {
    symbols: Vec<HeaderSymbol>,
    types: Vec<HeaderTypeKind>,
}

impl HeaderFile {
    /// Convert module to header file by extracting all exported symbols
    /// and types into parseable string representations.
    pub fn from_module(ctx: &Context, module: &Module) -> HeaderFile {
        let mut mappings = HashMap::new();
        let mut types = Vec::new();

        // Get all type ids used in this module
        let all_types_ids = module
            .exports()
            .values()
            .map(|id| ctx.types.get_all_references(ctx.symbols.get(*id).ty))
            .flatten()
            .collect::<HashSet<_>>();

        // Create a map from TypeId to HeaderTypeKind to store in the header file
        for ty in all_types_ids {
            let kind = real_to_header(ctx, &ctx.types.lookup(ty).kind);
            let id = types.len();
            types.push(kind);
            mappings.insert(ty, id);
        }

        // Convert all symbols to HeaderSymbol
        let symbols = module
            .exports()
            .iter()
            .map(|(_, id)| {
                let symbol = ctx.symbols.get(*id);

                HeaderSymbol {
                    name: symbol.name.clone(),
                    ty: *mappings
                        .get(&symbol.ty)
                        .expect("all types should be mapped"),
                    kind: symbol.kind.clone(),
                    no_mangle: symbol.no_mangle,
                    is_extern: symbol.is_extern(),
                }
            })
            .collect();

        HeaderFile { symbols, types }
    }

    /// Parse this headers symbols content and create module.
    pub fn to_module(self, ctx: &mut Context, modpath: ModulePath) -> Result<CreateModule, String> {
        // Create map of header id to real id (HeaderTypeId -> TypeId)
        let mappings = self
            .types
            .iter()
            .enumerate()
            .map(|(hid, hkind)| (hid, header_to_real(ctx, hkind)))
            .collect::<HashMap<_, _>>();

        // Convert header symbols to Symbols
        let symbols = self
            .symbols
            .into_iter()
            .map(|s| {
                let create_symbol = CreateSymbol {
                    kind: s.kind,
                    name: s.name,
                    no_mangle: s.no_mangle,
                    is_exported: true, // Always true for imported symbols
                    ty: *mappings.get(&s.ty).expect("mapping not found"),
                    origin: match s.is_extern {
                        true => SymbolOrigin::Extern,
                        false => SymbolOrigin::Library(modpath.clone()),
                    },
                };

                let name = create_symbol.name.clone();
                let id = ctx.symbols.add(create_symbol);
                let modsym = ModuleSymbol {
                    id,
                    kind: ModuleSymbolKind::Exported,
                };
                (name, modsym)
            })
            .collect::<HashMap<_, _>>()
            .into();

        Ok(CreateModule {
            modpath,
            kind: ModuleKind::External,
            symbols,
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
    // If this symbols is external it needs to be kept that way to
    // not mangle the symbol name when loading this module.
    is_extern: bool,
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
fn real_to_header(ctx: &Context, kind: &TypeKind) -> HeaderTypeKind {
    match kind {
        TypeKind::Primitive(p) => HeaderTypeKind::Primitive(p.into()),
        TypeKind::Array(id) => HeaderTypeKind::Array(boxed_kind(ctx, *id)),
        TypeKind::Pointer(id) => HeaderTypeKind::Pointer(boxed_kind(ctx, *id)),
        TypeKind::Alias(id) => HeaderTypeKind::Alias(boxed_kind(ctx, *id)),
        TypeKind::Unique(id) => HeaderTypeKind::Unique(boxed_kind(ctx, *id)),
        TypeKind::Function(func) => {
            let params = func
                .params
                .iter()
                .map(|id| real_to_header(ctx, &ctx.types.lookup(*id).kind))
                .collect();
            let ret = boxed_kind(ctx, func.ret);
            HeaderTypeKind::Function(params, ret)
        }
    }
}

fn boxed_kind(ctx: &Context, ty: TypeId) -> Box<HeaderTypeKind> {
    Box::new(real_to_header(ctx, &ctx.types.lookup(ty).kind))
}

/// Convert header type kind to real type kinds id.
fn header_to_real(ctx: &mut Context, kind: &HeaderTypeKind) -> TypeId {
    let typekind = match kind {
        HeaderTypeKind::Primitive(p) => TypeKind::Primitive(p.into()),
        HeaderTypeKind::Array(inner) => TypeKind::Array(header_to_real(ctx, inner)),
        HeaderTypeKind::Pointer(inner) => TypeKind::Pointer(header_to_real(ctx, inner)),
        HeaderTypeKind::Alias(inner) => TypeKind::Alias(header_to_real(ctx, inner)),
        HeaderTypeKind::Unique(inner) => TypeKind::Unique(header_to_real(ctx, inner)),
        HeaderTypeKind::Function(params, ret) => {
            let param_ids = params.iter().map(|p| header_to_real(ctx, p)).collect();
            let ret_id = header_to_real(ctx, ret);
            TypeKind::Function(crate::types::FunctionType {
                params: param_ids,
                ret: ret_id,
            })
        }
    };
    ctx.types.get_or_intern(typekind)
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
