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
    /// Top level symbol mappings.
    namespaces: HashMap<String, Namespace>,
}

#[derive(Clone)]
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
    pub fn set_symbol(&mut self, name: String, ty: TypeId, exported: bool) -> Result<(), String> {
        self.symbols
            .insert(name, Symbol { ty, exported })
            .map_or(Ok(()), |_| Err(format!("already declared")))
    }

    /// Get top level named type
    pub fn get_symbol(&self, name: &str) -> Result<TypeId, String> {
        self.symbols
            .get(name)
            .map_or(Err("not declared".to_string()), |s| Ok(s.ty))
    }

    /// Get an owned list of all exported symbols from this context.
    pub fn exported_symbols(&self) -> Vec<(String, TypeKind)> {
        self.symbols
            .iter()
            .filter(|s| (s.1).exported)
            .map(|s| (s.0.clone(), self.lookup(s.1.ty).kind.clone()))
            .collect::<Vec<_>>()
    }

    pub fn set_namespace(&mut self, ns: Namespace) -> Result<(), String> {
        self.namespaces
            .insert(ns.name.clone(), ns)
            .map_or(Ok(()), |_| Err(format!("already declared")))
    }

    pub fn get_namespace(&self, name: &str) -> Result<&Namespace, String> {
        self.namespaces
            .get(name)
            .map_or(Err("not declared".to_string()), |s| Ok(s))
    }

    /// Get the string representation of a type for errors or logging.
    pub fn to_string(&self, id: TypeId) -> String {
        match &self.lookup(id).kind {
            TypeKind::Primitive(p) => format!("{p}"),
            TypeKind::Array(inner) => format!("[]{}", self.to_string(*inner)),
            TypeKind::Pointer(inner) => format!("*{}", self.to_string(*inner)),
            TypeKind::Alias(id) => format!("{}", self.to_string(*id)),
            TypeKind::Unique(id) => format!("{}", self.to_string(*id)),
            TypeKind::Function(f) => {
                let params_str = f
                    .params
                    .iter()
                    .map(|p| self.to_string(*p))
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_str = self.to_string(f.ret);
                format!("func ({}) {}", params_str, ret_str)
            }
        }
    }

    pub fn to_string_debug(&self, id: TypeId) -> String {
        match &self.lookup(id).kind {
            TypeKind::Primitive(p) => format!("{p}"),
            TypeKind::Array(inner) => format!("Array<{}>", self.to_string(*inner)),
            TypeKind::Pointer(inner) => format!("Pointer<{}>", self.to_string(*inner)),
            TypeKind::Alias(id) => format!("Alias({})", self.to_string(*id)),
            TypeKind::Unique(id) => format!("Unique({})", self.to_string(*id)),
            TypeKind::Function(f) => {
                let params_str = f
                    .params
                    .iter()
                    .map(|p| self.to_string(*p))
                    .collect::<Vec<_>>()
                    .join(", ");

                let ret_str = self.to_string(f.ret);
                format!("func ({}) {} ({})", params_str, ret_str, f.origin)
            }
        }
    }

    /// Print a string dump of all type and symbol mappings.
    pub fn dump_context_string(&self) {
        let mut s = String::new();

        s += "| TYPES\n";
        s += "|-------------------------------\n";
        for i in 0..self.types.len() {
            s += &format!("| {:<3} {}\n", i, self.to_string_debug(i));
        }

        s += "| \n";
        s += "| SYMBOLS\n";
        s += "|-------------------------------\n";
        for sym in &self.symbols {
            s += &format!(
                "| {:<10} {:<3} {}\n",
                sym.0,
                (sym.1).ty,
                if (sym.1).exported { "(public)" } else { "" },
            );
        }

        println!("{}", s);
    }
}
