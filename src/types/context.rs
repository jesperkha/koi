use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::{
    ast::{Node, NodeId, TypeNode},
    token::{Token, TokenKind},
    types::{PrimitiveType, Type, TypeId, TypeKind},
};

/// Context for type lookups.
pub struct TypeContext {
    /// List of type information. Each `TypeId` maps
    /// to a `Type` by indexing into this vector.
    types: Vec<Type>,
    /// Map type kinds to their unique type id.
    cache: HashMap<TypeKind, TypeId>,
    /// Map AST nodes to their evaluated type.
    nodes: HashMap<NodeId, TypeId>,
    /// Map declared type names in current context.
    named: HashMap<String, TypeId>,
}

impl TypeContext {
    pub fn new() -> Self {
        let mut s = Self {
            types: Vec::new(),
            cache: HashMap::new(),
            named: HashMap::new(),
            nodes: HashMap::new(),
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

    /// Internalize a node and its evaluated type.
    pub fn intern_node(&mut self, node: &dyn Node, ty: TypeId) {
        assert!(!self.nodes.contains_key(&node.id()), "duplicate node id");
        self.nodes.insert(node.id(), ty);
    }

    pub fn get_node(&self, node: &dyn Node) -> TypeId {
        self.nodes
            .get(&node.id())
            .expect(format!("node id {} not in map", node.id()).as_str())
            .clone()
    }

    /// Shorthand for getting a primitive type id.
    pub fn primitive(&mut self, kind: PrimitiveType) -> TypeId {
        self.cache
            .get(&TypeKind::Primitive(kind))
            .expect("all primitive types must be assigned at init")
            .clone()
    }

    /// Declare named type. Interns the type if new. Returns the type id.
    pub fn declare(&mut self, name: String, kind: TypeKind) -> TypeId {
        let id = self.get_or_intern(kind);
        self.named.insert(name, id);
        id
    }

    /// Get named type from declaration.
    pub fn get_declared(&self, name: String) -> Option<&Type> {
        self.named.get(&name).map(|id| self.lookup(*id))
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
        // Illegal state if id is not known
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

    /// Returns the type id for a given ast type node (type literal). Refers to internal
    /// lookup for named types, aliases etc. Empty return means type does not exist.
    pub fn resolve_ast_node_type(&mut self, node: &TypeNode) -> Option<TypeId> {
        match node {
            TypeNode::Primitive(tok) => {
                let prim_kind = Self::ast_primitive_to_type_primitive(tok);
                Some(self.get_or_intern(TypeKind::Primitive(prim_kind)))
            }

            TypeNode::Ident(tok) => match &tok.kind {
                TokenKind::IdentLit(name) => self.get_declared(name.to_string()).map(|t| t.id),
                _ => panic!("identifier type node did not have a IdentLit token"),
            },
        }
    }

    /// Convert AST primitive literal to primitive type.
    fn ast_primitive_to_type_primitive(token: &Token) -> PrimitiveType {
        match token.kind {
            TokenKind::BoolType => PrimitiveType::Bool,
            TokenKind::ByteType => PrimitiveType::Byte,

            // Builtin 'aliases'
            TokenKind::IntType => PrimitiveType::Int64,
            TokenKind::FloatType => PrimitiveType::Float64,
            _ => panic!("unknown TypeNode::Primitive kind"),
        }
    }
}
