use crate::types::{FunctionType, NO_TYPE, PrimitiveType, Type, TypeId, TypeKind};
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;

pub struct TypeInterner {
    types: Vec<Type>,
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

    pub fn get_or_intern(&mut self, kind: TypeKind) -> TypeId {
        if let Some(&id) = self.cache.get(&kind) {
            return id;
        }
        self.intern(kind)
    }

    pub fn primitive(&self, kind: PrimitiveType) -> TypeId {
        *self
            .cache
            .get(&TypeKind::Primitive(kind))
            .expect("all primitive types must be assigned at init")
    }

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

    pub fn lookup(&self, id: TypeId) -> &Type {
        assert_ne!(id, NO_TYPE);
        assert!(id < self.types.len());
        &self.types[id]
    }

    pub fn get(&self, id: TypeId) -> Option<&Type> {
        self.types.get(id)
    }

    pub fn try_function(&self, id: TypeId) -> Option<&FunctionType> {
        if let Some(ty) = self.get(id)
            && let TypeKind::Function(func) = &ty.kind
        {
            return Some(func);
        }
        None
    }

    pub fn resolve(&self, id: TypeId) -> TypeId {
        match &self.lookup(id).kind {
            TypeKind::Alias(target) => self.resolve(*target),
            _ => id,
        }
    }

    pub fn deep_resolve(&self, id: TypeId) -> TypeId {
        match &self.lookup(id).kind {
            TypeKind::Alias(target) | TypeKind::Unique(_, target) => self.resolve(*target),
            _ => id,
        }
    }

    pub fn inner_kind(&self, id: TypeId) -> TypeId {
        match &self.lookup(id).kind {
            TypeKind::Alias(underlying) | TypeKind::Unique(_, underlying) => {
                self.inner_kind(*underlying)
            }
            _ => id,
        }
    }

    pub fn equivalent(&self, a: TypeId, b: TypeId) -> bool {
        self.resolve(a) == self.resolve(b)
    }

    pub fn is_number(&self, id: TypeId) -> bool {
        [
            self.primitive(PrimitiveType::F32),
            self.primitive(PrimitiveType::F64),
            self.primitive(PrimitiveType::I8),
            self.primitive(PrimitiveType::I16),
            self.primitive(PrimitiveType::I32),
            self.primitive(PrimitiveType::I64),
            self.primitive(PrimitiveType::U8),
            self.primitive(PrimitiveType::U16),
            self.primitive(PrimitiveType::U32),
            self.primitive(PrimitiveType::U64),
        ]
        .contains(&id)
    }

    pub fn void(&self) -> TypeId {
        self.primitive(PrimitiveType::Void)
    }

    pub fn void_type(&self) -> Type {
        Type {
            kind: TypeKind::Primitive(PrimitiveType::Void),
            id: self.primitive(PrimitiveType::Void),
        }
    }

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
                | TypeKind::Unique(_, inner) => stack.push(*inner),
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

    pub fn type_to_string(&self, id: TypeId) -> String {
        match &self.lookup(id).kind {
            TypeKind::Primitive(p) => format!("{p}"),
            TypeKind::Array(inner) => format!("[]{}", self.type_to_string(*inner)),
            TypeKind::Pointer(inner) => format!("*{}", self.type_to_string(*inner)),
            TypeKind::Alias(id) => self.type_to_string(*id).to_string(),
            TypeKind::Unique(name, _) => name.into(),
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
            TypeKind::Unique(name, id) => format!("Unique({name} {})", self.type_to_string(*id)),
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
