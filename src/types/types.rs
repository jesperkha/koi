use std::{fmt, hash::Hash};
use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Array(TypeId),
    Pointer(TypeId),
    Alias(TypeId),  // Refers to another type definition
    Unique(TypeId), // Distinct nominal type

    /// List of parameter types and a return
    /// type (void for no return)
    Function(FunctionType),
}

// TODO: add positional info to type object to point to related declarations in errors

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type {
    pub kind: TypeKind,
    pub id: TypeId, // Unique identifier for interning/comparison
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
    Byte,
    String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FunctionOrigin {
    /// Full module path
    Module(String),
    Extern,
}

impl fmt::Display for FunctionOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FunctionOrigin::Module(s) => &s,
                FunctionOrigin::Extern => "extern",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    pub params: Vec<TypeId>,
    pub ret: TypeId,
}

pub type TypeId = usize; // Unique identifier

/// Get the id of invalid types (not assigned yet).
pub fn no_type() -> TypeId {
    return usize::MAX;
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}
