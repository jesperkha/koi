use std::{collections::HashMap, fmt};

use crate::{
    context::Context,
    types::{self, TypeId},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IRType {
    Primitive(Primitive),
    Function(Vec<IRType>, Box<IRType>),
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
        }
    }
}

pub type IRTypeId = usize;

pub struct IRTypeInterner {
    types: Vec<IRType>,
    cache: HashMap<IRType, IRTypeId>,
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
        self.types.push(ty.clone()); // TODO: need this clone?
        self.cache.insert(ty, id);
        id
    }

    pub fn to_ir_type_list(&mut self, ctx: &Context, list: &[TypeId]) -> Vec<IRTypeId> {
        list.iter().map(|ty| self.to_ir(ctx, *ty)).collect()
    }

    pub fn to_ir(&mut self, ctx: &Context, id: TypeId) -> IRTypeId {
        self.get_or_intern(self.to_ir_type(ctx, id))
    }

    fn to_ir_type(&self, ctx: &Context, id: TypeId) -> IRType {
        let id = ctx.types.deep_resolve(id);
        let ty = ctx.types.lookup(id);

        match &ty.kind {
            types::TypeKind::Primitive(p) => IRType::Primitive(p.clone().into()),
            types::TypeKind::Function(f) => IRType::Function(
                f.params.iter().map(|p| self.to_ir_type(ctx, *p)).collect(),
                Box::new(self.to_ir_type(ctx, f.ret)),
            ),
            _ => panic!("unhandled kind {:?}", ty.kind),
        }
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
}
