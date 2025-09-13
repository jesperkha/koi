use std::fmt;
use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Array(Box<Type>),
    Pointer(Box<Type>),
    Alias(TypeId),  // Refers to another type definition
    Unique(TypeId), // Distinct nominal type
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type {
    pub kind: TypeKind,
    pub id: TypeId, // Unique identifier for interning/comparison
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum PrimitiveType {
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
}

pub type TypeId = usize; // Unique identifier

/// Get the id of invalid types (not assigned yet).
pub fn no_type() -> TypeId {
    return usize::MAX;
}

/// Get the id of the void type (no type information).
pub fn void_type() -> TypeId {
    return usize::MAX - 1;
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Primitive(p) => write!(f, "{p}"),
            TypeKind::Array(inner) => write!(f, "[]{}", inner),
            TypeKind::Pointer(inner) => write!(f, "*{}", inner),
            TypeKind::Alias(id) => write!(f, "Alias({id})"),
            TypeKind::Unique(id) => write!(f, "Unique({id})"),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // You could include the type ID if useful for debugging
        write!(f, "{}#{}", self.kind, self.id)
    }
}
