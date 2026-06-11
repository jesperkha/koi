use std::{
    collections::{HashMap, HashSet},
    fs::read,
};

use serde::{Deserialize, Serialize};

use koi_common::config::Config;

use crate::{
    context::{Context, CreateModule, CreateSymbol},
    module::{
        ImportPath, Module, ModuleId, ModuleKind, ModulePath, ModuleSymbol, ModuleSymbolKind,
        SymbolKind, SymbolOrigin,
    },
    types::{PrimitiveType, TypeId, TypeKind},
};

pub fn create_header_file(ctx: &Context, id: ModuleId) -> Result<Vec<u8>, String> {
    let module = ctx.modules.get(id);
    let header = HeaderFile::from_module(ctx, module);
    postcard::to_stdvec(&header).map_err(|e| e.to_string())
}

pub fn read_header_file(
    ctx: &mut Context,
    modpath: ModulePath,
    bytes: &[u8],
) -> Result<CreateModule, String> {
    let header: HeaderFile = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    header.into_module(ctx, modpath)
}

pub fn dump_header_symbols(filepath: &str) -> Result<String, String> {
    let modpath = ModulePath::from(ImportPath::from("header"));
    let bytes = read(filepath).map_err(|e| format!("failed to read header file: {}", e))?;
    let mut ctx = Context::new(Config::normal());
    let module = read_header_file(&mut ctx, modpath, &bytes)?;
    Ok(module.symbols.dump(&ctx, filepath))
}

#[derive(Debug, Serialize, Deserialize)]
struct HeaderFile {
    symbols: Vec<HeaderSymbol>,
    types: Vec<HeaderTypeKind>,
}

impl HeaderFile {
    pub fn from_module(ctx: &Context, module: &Module) -> HeaderFile {
        let mut mappings = HashMap::new();
        let mut types = Vec::new();

        let all_types_ids = module
            .exports()
            .values()
            .flat_map(|id| ctx.types.get_all_references(ctx.symbols.get(*id).ty))
            .collect::<HashSet<_>>();

        for ty in all_types_ids {
            let kind = real_to_header(ctx, &ctx.types.lookup(ty).kind);
            let id = types.len();
            types.push(kind);
            mappings.insert(ty, id);
        }

        let symbols = module
            .exports()
            .values()
            .map(|id| {
                let symbol = ctx.symbols.get(*id);

                HeaderSymbol {
                    name: symbol.name.clone(),
                    alias: symbol.alias.clone(),
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

    pub fn into_module(
        self,
        ctx: &mut Context,
        modpath: ModulePath,
    ) -> Result<CreateModule, String> {
        let mappings = self
            .types
            .iter()
            .enumerate()
            .map(|(hid, hkind)| (hid, header_to_real(ctx, hkind)))
            .collect::<HashMap<_, _>>();

        let symbols = self
            .symbols
            .into_iter()
            .map(|s| {
                let create_symbol = CreateSymbol {
                    alias: s.alias,
                    kind: s.kind,
                    name: s.name,
                    no_mangle: s.no_mangle,
                    is_exported: true,
                    ty: *mappings.get(&s.ty).expect("mapping not found"),
                    origin: match s.is_extern {
                        true => SymbolOrigin::Extern,
                        false => SymbolOrigin::Library(modpath.clone()),
                    },
                };

                let name = create_symbol
                    .alias
                    .as_ref()
                    .map_or(&create_symbol.name, |alias| alias)
                    .clone();
                let id = ctx.symbols.add(create_symbol);
                let modsym = ModuleSymbol {
                    id,
                    exported: true,
                    kind: ModuleSymbolKind::Module,
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

#[derive(Debug, Serialize, Deserialize)]
struct HeaderSymbol {
    name: String,
    alias: Option<String>,
    ty: usize,
    kind: SymbolKind,
    no_mangle: bool,
    is_extern: bool,
}

#[derive(Debug, Serialize, Deserialize)]
enum HeaderTypeKind {
    Primitive(HeaderPrimitiveType),
    Array(Box<HeaderTypeKind>),
    Pointer(Box<HeaderTypeKind>),
    Alias(Box<HeaderTypeKind>),
    Unique(String, Box<HeaderTypeKind>),
    Function(Vec<HeaderTypeKind>, Box<HeaderTypeKind>),
}

fn real_to_header(ctx: &Context, kind: &TypeKind) -> HeaderTypeKind {
    match kind {
        TypeKind::Primitive(p) => HeaderTypeKind::Primitive(p.into()),
        TypeKind::Array(id) => HeaderTypeKind::Array(boxed_kind(ctx, *id)),
        TypeKind::Pointer(id) => HeaderTypeKind::Pointer(boxed_kind(ctx, *id)),
        TypeKind::Alias(id) => HeaderTypeKind::Alias(boxed_kind(ctx, *id)),
        TypeKind::Unique(name, id) => HeaderTypeKind::Unique(name.into(), boxed_kind(ctx, *id)),
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

fn header_to_real(ctx: &mut Context, kind: &HeaderTypeKind) -> TypeId {
    let typekind = match kind {
        HeaderTypeKind::Primitive(p) => TypeKind::Primitive(p.into()),
        HeaderTypeKind::Array(inner) => TypeKind::Array(header_to_real(ctx, inner)),
        HeaderTypeKind::Pointer(inner) => TypeKind::Pointer(header_to_real(ctx, inner)),
        HeaderTypeKind::Alias(inner) => TypeKind::Alias(header_to_real(ctx, inner)),
        HeaderTypeKind::Unique(name, inner) => {
            TypeKind::Unique(name.into(), header_to_real(ctx, inner))
        }
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
