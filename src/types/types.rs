use std::collections::HashMap;
use strum::IntoEnumIterator;
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
}

pub type TypeId = usize; // Unique identifier

/// Context for type lookups.
pub struct TypeContext {
    types: Vec<Type>,
    cache: HashMap<TypeKind, TypeId>,
}

impl TypeContext {
    pub fn new() -> Self {
        let mut s = Self {
            types: Vec::new(),
            cache: HashMap::new(),
        };

        s.init_universe();
        s
    }

    /// Returns the unique type id for the given kind.
    /// Stores the type in context if not seen before.
    pub fn get_or_intern(&mut self, kind: TypeKind) -> TypeId {
        if let Some(&id) = self.cache.get(&kind) {
            return id;
        }
        self.intern(kind)
    }

    fn intern(&mut self, kind: TypeKind) -> TypeId {
        let id = self.types.len();
        let typ = Type {
            kind: kind.clone(),
            id,
        };
        self.types.push(typ);
        self.cache.insert(kind, id);
        id
    }

    pub fn lookup(&self, id: TypeId) -> &Type {
        // Illegal state if id is not known
        assert!(id <= self.types.len());
        &self.types[id]
    }

    /// Resolve a type to its type id by removing any aliasing.
    pub fn resolve(&self, id: TypeId) -> TypeId {
        let typ = self.lookup(id);
        match &typ.kind {
            TypeKind::Alias(target) => self.resolve(*target),
            _ => id,
        }
    }

    /// Get a types internal kind. Resolves array item types, pointer target
    /// types, and unique types underlying kind.
    pub fn inner_kind(&self, id: TypeId) -> TypeId {
        let typ = self.lookup(id);
        match &typ.kind {
            TypeKind::Alias(underlying) => self.inner_kind(*underlying),
            _ => id,
        }
    }

    pub fn equivalent(&self, a: TypeId, b: TypeId) -> bool {
        self.resolve(a) == self.resolve(b)
    }

    /// Initialize all built-in types and aliases known before beginning type check.
    fn init_universe(&mut self) {
        for t in PrimitiveType::iter() {
            self.intern(TypeKind::Primitive(t));
        }

        // 'int' and 'float' aliases
        let int64_id = self.get_or_intern(TypeKind::Primitive(PrimitiveType::Uint64));
        self.intern(TypeKind::Alias(int64_id));

        let float64_id = self.get_or_intern(TypeKind::Primitive(PrimitiveType::Float64));
        self.intern(TypeKind::Alias(float64_id));
    }
}
