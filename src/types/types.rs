use std::{collections::HashMap, fmt, hash::Hash};
use strum_macros::EnumIter;

use crate::types::{Exports, TypeContext};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Primitive(PrimitiveType),
    Array(TypeId),
    Pointer(TypeId),
    Alias(TypeId),  // Refers to another type definition
    Unique(TypeId), // Distinct nominal type

    /// List of parameter types and a return
    /// type (void for no return)
    Function(Vec<TypeId>, TypeId),

    Namespace(NamespaceType),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceType {
    pub name: String,
    pub symbols: HashMap<String, TypeId>,
}

impl NamespaceType {
    pub fn new(name: String, exports: &Exports, ctx: &mut TypeContext) -> Self {
        let mut ns = NamespaceType {
            name,
            symbols: HashMap::new(),
        };

        for (name, kind) in exports.symbols() {
            let id = ctx.get_or_intern(kind.clone());
            ns.symbols.insert(name.to_string(), id);
        }

        ns
    }
}

impl Hash for NamespaceType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
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
