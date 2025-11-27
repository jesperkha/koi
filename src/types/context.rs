use std::{collections::HashMap, str};
use strum::IntoEnumIterator;

use crate::types::{Namespace, PrimitiveType, Type, TypeId, TypeKind, no_type};

/// Context for type lookups.
pub struct TypeContext {
    /// List of type information. Each `TypeId` maps
    /// to a `Type` by indexing into this vector.
    types: Vec<Type>,
    /// Map type kinds to their unique type id.
    cache: HashMap<TypeKind, TypeId>,
    /// Top level symbol mappings.
    symbols: HashMap<String, Symbol>,
    /// Map of namespaces imported into this context
    namespaces: HashMap<String, Namespace>,
}

pub struct Symbol {
    pub ty: TypeId,
    pub exported: bool,
}

impl TypeContext {
    // TODO: accept exported symbols and intern at init
    pub fn new() -> Self {
        let mut s = Self {
            types: Vec::new(),
            cache: HashMap::new(),
            symbols: HashMap::new(),
            namespaces: HashMap::new(),
        };

        for t in PrimitiveType::iter() {
            s.intern(TypeKind::Primitive(t));
        }

        s
    }

    /// Add new namespace to this context. Returns error if namespace with that name already exists.
    pub fn add_namespace(&mut self, namespace: Namespace) -> Result<(), String> {
        self.namespaces
            .insert(namespace.name.clone(), namespace)
            .map_or(Ok(()), |n| {
                Err(format!("namespace '{}' already imported", n.name))
            })
    }

    /// Get the string representation of a type for errors or logging.
    pub fn to_string(&self, id: TypeId) -> String {
        match &self.lookup(id).kind {
            TypeKind::Primitive(p) => format!("{p}"),
            TypeKind::Array(inner) => format!("[]{}", self.to_string(*inner)),
            TypeKind::Pointer(inner) => format!("*{}", self.to_string(*inner)),
            TypeKind::Alias(id) => format!("Alias({})", self.to_string(*id)),
            TypeKind::Unique(id) => format!("Unique({})", self.to_string(*id)),
            TypeKind::Function(params, ret) => {
                let params_str = params
                    .iter()
                    .map(|p| self.to_string(*p))
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_str = self.to_string(*ret);
                format!("func ({}) {}", params_str, ret_str)
            }
        }
    }

    /// Returns the unique type id for the given kind.
    /// Stores the type in context if not seen before.
    pub fn get_or_intern(&mut self, kind: TypeKind) -> TypeId {
        if let Some(&id) = self.cache.get(&kind) {
            return id;
        }
        self.intern(kind)
    }

    /// Shorthand for getting a primitive type id.
    pub fn primitive(&self, kind: PrimitiveType) -> TypeId {
        self.cache
            .get(&TypeKind::Primitive(kind))
            .expect("all primitive types must be assigned at init")
            .clone()
    }

    /// Shorthand for getting the Type of a primitive kind.
    pub fn primitive_type(&mut self, kind: PrimitiveType) -> &Type {
        let id = self.primitive(kind);
        self.lookup(id)
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

    /// Get the full type information for a given type id.
    pub fn lookup(&self, id: TypeId) -> &Type {
        // Illegal state if id is noType or not known
        assert_ne!(id, no_type());
        assert!(id <= self.types.len());
        &self.types[id]
    }

    /// Resolve a type to its type id for comparisons. Removes any aliasing.
    /// Does not remove unique type aliases like `inner_kind()`.
    pub fn resolve(&self, id: TypeId) -> TypeId {
        match &self.lookup(id).kind {
            TypeKind::Alias(target) => self.resolve(*target),
            _ => id,
        }
    }

    /// Resolve to base type, removes aliasing and unique types.
    pub fn deep_resolve(&self, id: TypeId) -> TypeId {
        match &self.lookup(id).kind {
            TypeKind::Alias(target) | TypeKind::Unique(target) => self.resolve(*target),
            _ => id,
        }
    }

    /// Get a types internal kind. Resolves array item types, pointer target
    /// types, and unique types underlying kind. Do not use for general type comparisons.
    pub fn inner_kind(&self, id: TypeId) -> TypeId {
        match &self.lookup(id).kind {
            TypeKind::Alias(underlying) | TypeKind::Unique(underlying) => {
                self.inner_kind(*underlying)
            }
            _ => id,
        }
    }

    /// Tests if two types are equivalent (resolves any aliasing).
    pub fn equivalent(&self, a: TypeId, b: TypeId) -> bool {
        self.resolve(a) == self.resolve(b)
    }

    /// Shorthand for getting void type
    pub fn void(&mut self) -> TypeId {
        self.primitive(PrimitiveType::Void)
    }

    // Shorthand for getting the Type of void.
    pub fn void_type(&mut self) -> Type {
        Type {
            kind: TypeKind::Primitive(PrimitiveType::Void),
            id: self.primitive(PrimitiveType::Void),
        }
    }

    /// Set top level named type
    pub fn set_symbol(&mut self, name: String, ty: TypeId, exported: bool) {
        self.symbols.insert(name, Symbol { ty, exported });
    }

    /// Get top level named type
    pub fn get_symbol(&mut self, name: &str) -> Result<TypeId, String> {
        self.symbols
            .get(name)
            .map_or(Err("not declared".to_string()), |s| Ok(s.ty))
    }
}
