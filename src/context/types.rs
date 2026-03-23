use crate::types::{FunctionType, NO_TYPE, PrimitiveType, Type, TypeId, TypeKind};
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;

/// Context for type lookups.
pub struct TypeInterner {
    /// List of type information. Each `TypeId` maps
    /// to a `Type` by indexing into this vector.
    types: Vec<Type>,
    /// Map type kinds to their unique type id.
    cache: HashMap<TypeKind, TypeId>,
}

impl Default for TypeInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeInterner {
    pub fn new() -> Self {
        let mut s = Self {
            types: Vec::new(),
            cache: HashMap::new(),
        };

        for t in PrimitiveType::iter() {
            s.intern(TypeKind::Primitive(t));
        }

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

    /// Shorthand for getting a primitive type id.
    pub fn primitive(&self, kind: PrimitiveType) -> TypeId {
        *self.cache
            .get(&TypeKind::Primitive(kind))
            .expect("all primitive types must be assigned at init")
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
        assert_ne!(id, NO_TYPE);
        assert!(id < self.types.len());
        &self.types[id]
    }

    /// Get a Type
    pub fn get(&self, id: TypeId) -> Option<&Type> {
        self.types.get(id)
    }

    /// Try to get the inner FunctionType of this type id.
    pub fn try_function(&self, id: TypeId) -> Option<&FunctionType> {
        if let Some(ty) = self.get(id)
            && let TypeKind::Function(func) = &ty.kind {
                return Some(func);
            }
        None
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
    pub fn void(&self) -> TypeId {
        self.primitive(PrimitiveType::Void)
    }

    // Shorthand for getting the Type of void.
    pub fn void_type(&self) -> Type {
        Type {
            kind: TypeKind::Primitive(PrimitiveType::Void),
            id: self.primitive(PrimitiveType::Void),
        }
    }

    /// Returns a list of all type ids constructing this one.
    pub fn get_all_references(&self, ty: TypeId) -> HashSet<TypeId> {
        let mut refs = HashSet::new();
        let mut stack = vec![ty];

        while let Some(current) = stack.pop() {
            if refs.contains(&current) {
                continue;
            }

            refs.insert(current);
            match &self.lookup(current).kind {
                TypeKind::Array(inner)
                | TypeKind::Pointer(inner)
                | TypeKind::Alias(inner)
                | TypeKind::Unique(inner) => stack.push(*inner),
                TypeKind::Function(func) => {
                    for param in &func.params {
                        stack.push(*param);
                    }
                    stack.push(func.ret);
                }
                TypeKind::Primitive(p) => {
                    refs.insert(self.primitive(p.clone()));
                }
            }
        }

        refs
    }

    /// Get the string representation of a type for errors or logging.
    pub fn type_to_string(&self, id: TypeId) -> String {
        match &self.lookup(id).kind {
            TypeKind::Primitive(p) => format!("{p}"),
            TypeKind::Array(inner) => format!("[]{}", self.type_to_string(*inner)),
            TypeKind::Pointer(inner) => format!("*{}", self.type_to_string(*inner)),
            TypeKind::Alias(id) => self.type_to_string(*id).to_string(),
            TypeKind::Unique(id) => self.type_to_string(*id).to_string(),
            TypeKind::Function(f) => {
                let params_str = f
                    .params
                    .iter()
                    .map(|p| self.type_to_string(*p))
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_str = self.type_to_string(f.ret);
                format!("func ({}) {}", params_str, ret_str)
            }
        }
    }

    pub fn type_to_string_debug(&self, id: TypeId) -> String {
        match &self.lookup(id).kind {
            TypeKind::Primitive(p) => format!("{p}"),
            TypeKind::Array(inner) => format!("Array<{}>", self.type_to_string(*inner)),
            TypeKind::Pointer(inner) => format!("Pointer<{}>", self.type_to_string(*inner)),
            TypeKind::Alias(id) => format!("Alias({})", self.type_to_string(*id)),
            TypeKind::Unique(id) => format!("Unique({})", self.type_to_string(*id)),
            TypeKind::Function(f) => {
                let params_str = f
                    .params
                    .iter()
                    .map(|p| self.type_to_string(*p))
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_str = self.type_to_string(f.ret);
                format!("func ({}) {}", params_str, ret_str)
            }
        }
    }

    /// Print a string dump of all type and symbol mappings.
    pub fn dump_context_string(&self) -> String {
        let mut s = String::new();

        s += "| Types\n";
        s += "|-------------------------------\n";
        for i in 0..self.types.len() {
            s += &format!("| {:<3} {}\n", i, self.type_to_string_debug(i));
        }

        s
    }
}
