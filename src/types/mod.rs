mod nodes;

pub use nodes::*;

use std::{fmt, hash::Hash};
use strum_macros::EnumIter;

pub type TypeId = usize; // Unique identifier

/// ID of invalid types (not assigned yet).
pub const NO_TYPE: TypeId = usize::MAX;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type {
    pub kind: TypeKind,
    pub id: TypeId, // Unique identifier for interning/comparison
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Array(TypeId),
    Pointer(TypeId),
    Alias(TypeId),          // Refers to another type definition
    Unique(String, TypeId), // Distinct nominal type with name

    /// List of parameter types and a return
    /// type (void for no return)
    Function(FunctionType),
}

pub enum CastKind {
    /// Bad cast, error
    InvalidCast,
    /// Both types are the same
    Identity,

    IntegerNarrowing,
    IntegerWidening,
    FloatWidening,
    FloatNarrowing,

    FloatToInt,
    IntToFloat,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum PrimitiveType {
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
    Byte, // TODO: remove in place of u8
    String,
}

impl PrimitiveType {
    pub fn bytes(&self) -> usize {
        match self {
            PrimitiveType::Void => 0,
            PrimitiveType::I8 | PrimitiveType::U8 | PrimitiveType::Bool | PrimitiveType::Byte => 1,
            PrimitiveType::I16 | PrimitiveType::U16 => 2,
            PrimitiveType::I32 | PrimitiveType::U32 | PrimitiveType::F32 => 4,
            PrimitiveType::I64 | PrimitiveType::U64 | PrimitiveType::F64 => 8,
            PrimitiveType::String => todo!(),
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(
            self,
            PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 | PrimitiveType::I64
        )
    }

    pub fn is_uint(&self) -> bool {
        matches!(
            self,
            PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 | PrimitiveType::U64
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, PrimitiveType::F32 | PrimitiveType::F64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub params: Vec<TypeId>,
    pub ret: TypeId,
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}
