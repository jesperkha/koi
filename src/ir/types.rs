use std::{collections::HashMap, fmt};

use crate::{
    context::Context,
    types::{self, TypeId},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IRType {
    Primitive(Primitive),
    Function(Vec<IRType>, Box<IRType>),
    Struct(String, Vec<(String, IRType)>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Primitive {
    Void,
    F32,
    F64,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    String,
}

impl From<types::PrimitiveType> for Primitive {
    fn from(value: types::PrimitiveType) -> Self {
        match value {
            types::PrimitiveType::Void => Self::Void,
            types::PrimitiveType::I8 => Self::I8,
            types::PrimitiveType::I16 => Self::I16,
            types::PrimitiveType::I32 => Self::I32,
            types::PrimitiveType::I64 => Self::I64,
            types::PrimitiveType::Byte | types::PrimitiveType::Bool | types::PrimitiveType::U8 => {
                Self::U8
            }
            types::PrimitiveType::U16 => Self::U16,
            types::PrimitiveType::U32 => Self::U32,
            types::PrimitiveType::U64 => Self::U64,
            types::PrimitiveType::F32 => Self::F32,
            types::PrimitiveType::F64 => Self::F64,
            types::PrimitiveType::String => Self::String,
        }
    }
}

const PTR_SIZE: usize = 8;

impl IRType {
    /// Get size of type in bytes
    pub fn size(&self) -> usize {
        match self {
            IRType::Primitive(primitive) => match primitive {
                Primitive::Void => 0,
                Primitive::U8 | Primitive::I8 => 1,
                Primitive::U16 | Primitive::I16 => 2,
                Primitive::F32 | Primitive::I32 | Primitive::U32 => 4,
                Primitive::F64 | Primitive::U64 | Primitive::I64 | Primitive::String => PTR_SIZE,
            },
            IRType::Function(_, _) => 8,
            IRType::Struct(_, fields) => fields.iter().map(|(_, ty)| ty.size()).sum(),
        }
    }
}

impl fmt::Display for IRType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IRType::Primitive(p) => write!(f, "{}", format!("{:?}", p).to_lowercase()),
            IRType::Function(params, ret) => write!(
                f,
                "func({})->{}",
                params
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                ret
            ),
            IRType::Struct(name, _) => write!(f, "{name}"),
        }
    }
}

pub type IRTypeId = usize;

pub struct IRTypeInterner {
    types: Vec<IRType>,
    cache: HashMap<IRType, IRTypeId>,
}

impl Default for IRTypeInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl IRTypeInterner {
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            cache: HashMap::new(),
        }
    }

    pub fn get_or_intern(&mut self, ty: IRType) -> IRTypeId {
        if let Some(id) = self.cache.get(&ty) {
            return *id;
        }
        let id = self.types.len();
        self.types.push(ty.clone());
        self.cache.insert(ty, id);
        id
    }

    pub fn to_ir_type_list(&mut self, ctx: &Context, list: &[TypeId]) -> Vec<IRTypeId> {
        list.iter().map(|ty| self.to_ir(ctx, *ty)).collect()
    }

    pub fn to_ir(&mut self, ctx: &Context, id: TypeId) -> IRTypeId {
        let ir_type = to_ir_type(ctx, id);
        self.intern_with_nested(ir_type)
    }

    // Intern a type, recursing into struct fields first so that nested struct
    // types always have their own interner entries before the containing struct.
    // This guarantees structs() yields types in dependency order for C codegen.
    fn intern_with_nested(&mut self, ty: IRType) -> IRTypeId {
        if let IRType::Struct(_, ref fields) = ty {
            for (_, field_ty) in fields {
                if matches!(field_ty, IRType::Struct(_, _)) {
                    self.intern_with_nested(field_ty.clone());
                }
            }
        }
        self.get_or_intern(ty)
    }

    pub fn dump(&self) -> String {
        let mut s = String::new();
        for (id, ty) in self.types.iter().enumerate() {
            s += &format!("{} -> {}\n", id, ty);
        }

        s
    }

    pub fn type_to_string(&self, id: IRTypeId) -> String {
        self.types[id].to_string()
    }

    pub fn sizeof(&self, id: IRTypeId) -> usize {
        let ty = &self.types[id];
        ty.size()
    }

    pub fn get(&self, id: IRTypeId) -> &IRType {
        &self.types[id]
    }

    pub fn structs(&self) -> impl Iterator<Item = (&str, &[(String, IRType)])> {
        self.types.iter().filter_map(|ty| {
            if let IRType::Struct(name, fields) = ty {
                Some((name.as_str(), fields.as_slice()))
            } else {
                None
            }
        })
    }
}

fn to_ir_type(ctx: &Context, id: TypeId) -> IRType {
    let id = ctx.types.deep_resolve(id);
    let ty = ctx.types.lookup(id);

    match &ty.kind {
        types::TypeKind::Primitive(p) => IRType::Primitive(p.clone().into()),
        types::TypeKind::Function(f) => IRType::Function(
            f.params.iter().map(|p| to_ir_type(ctx, *p)).collect(),
            Box::new(to_ir_type(ctx, f.ret)),
        ),
        types::TypeKind::Struct(s) => IRType::Struct(
            s.name.clone(),
            s.fields
                .iter()
                .map(|(name, tid)| (name.clone(), to_ir_type(ctx, *tid)))
                .collect(),
        ),
        _ => panic!("unhandled kind {:?}", ty.kind),
    }
}
