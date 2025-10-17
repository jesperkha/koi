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
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    Bool,
    Byte,
    String,
}

pub type TypeId = usize; // Unique identifier

/// Get the id of invalid types (not assigned yet).
pub fn no_type() -> TypeId {
    return usize::MAX;
}
