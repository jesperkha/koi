use crate::{
    ast::{Node, NodeId, Pos, TokenKind},
    types::{Type, TypeId, TypeKind},
};

pub trait TypedNode<'a> {
    /// Get the TypeKind of this node.
    fn kind(&'a self) -> &'a TypeKind;
    /// Get the unique TypeId for this node, only to be used within the same module.
    fn type_id(&self) -> TypeId;
}

pub trait Visitable {
    /// Accept visitor to inspect this node.
    fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T;
}

pub trait Visitor<T> {
    fn visit_func(&mut self, node: &FuncNode) -> T;
    fn visit_return(&mut self, node: &ReturnNode) -> T;
    fn visit_var_assign(&mut self, node: &VarAssignNode) -> T;
    fn visit_var_decl(&mut self, node: &VarDeclNode) -> T;
    fn visit_literal(&mut self, node: &LiteralNode) -> T;
    fn visit_extern(&mut self, node: &ExternNode) -> T;
    fn visit_call(&mut self, node: &CallNode) -> T;
    fn visit_member(&mut self, node: &MemberNode) -> T;
    fn visit_namespace_member(&mut self, node: &NamespaceMemberNode) -> T;
}

pub struct TypedAst {
    pub decls: Vec<Decl>,
}

impl TypedAst {
    pub fn new(decls: Vec<Decl>) -> Self {
        TypedAst { decls }
    }

    pub fn empty() -> Self {
        TypedAst { decls: vec![] }
    }
}

pub struct NodeMeta {
    pub id: NodeId,
    pub pos: Pos,
    pub end: Pos,
}

pub fn ast_node_to_meta(node: &dyn Node) -> NodeMeta {
    NodeMeta {
        id: node.id(),
        pos: node.pos().clone(),
        end: node.end().clone(),
    }
}

pub enum Decl {
    Extern(ExternNode),
    Func(FuncNode),
}

pub enum Stmt {
    Return(ReturnNode),
    VarDecl(VarDeclNode),
    VarAssign(VarAssignNode),
    ExprStmt(Expr),
}

pub enum Expr {
    Literal(LiteralNode),
    Call(CallNode),
    Member(MemberNode),
    NamespaceMember(NamespaceMemberNode),
}

pub struct FuncNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub name: String,
    pub params: Vec<String>,
    pub public: bool,
    pub body: Vec<Stmt>,
}

pub struct ExternNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub name: String,
}

pub struct ReturnNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub expr: Option<Expr>,
}

pub struct MemberNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub expr: Box<Expr>,
    pub field: String,
}

pub struct NamespaceMemberNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub name: String,
    pub field: String,
}

pub struct LiteralNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub kind: LiteralKind,
}

pub enum LiteralKind {
    Ident(String),
    String(String),
    Int(i64),
    Uint(u64),
    Float(f64),
    Bool(bool),
    Char(u8),
}

impl From<TokenKind> for LiteralKind {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::IdentLit(name) => LiteralKind::Ident(name),
            TokenKind::IntLit(n) => LiteralKind::Int(n),
            TokenKind::FloatLit(n) => LiteralKind::Float(n),
            TokenKind::StringLit(s) => LiteralKind::String(s),
            TokenKind::CharLit(c) => LiteralKind::Char(c),
            TokenKind::True => LiteralKind::Bool(true),
            TokenKind::False => LiteralKind::Bool(false),
            TokenKind::Null => todo!(),
            _ => panic!("unhandled token kind in conversion, {:?}", kind),
        }
    }
}

pub struct VarDeclNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub name: String,
    pub value: Expr,
}

pub struct VarAssignNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub lval: Expr,
    pub rval: Expr,
}

pub struct CallNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

impl Visitable for Decl {
    fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        match self {
            Decl::Func(node) => v.visit_func(node),
            Decl::Extern(node) => v.visit_extern(node),
        }
    }
}

impl Visitable for Stmt {
    fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        match self {
            Stmt::Return(node) => v.visit_return(node),
            Stmt::VarDecl(node) => v.visit_var_decl(node),
            Stmt::VarAssign(node) => v.visit_var_assign(node),
            Stmt::ExprStmt(node) => node.accept(v),
        }
    }
}

impl Visitable for Expr {
    fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Literal(node) => v.visit_literal(node),
            Expr::Call(node) => v.visit_call(node),
            Expr::Member(node) => v.visit_member(node),
            Expr::NamespaceMember(node) => v.visit_namespace_member(node),
        }
    }
}

impl Node for Decl {
    fn pos(&self) -> &Pos {
        match self {
            Decl::Extern(node) => &node.meta.pos,
            Decl::Func(node) => &node.meta.pos,
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Decl::Extern(node) => &node.meta.end,
            Decl::Func(node) => &node.meta.end,
        }
    }

    fn id(&self) -> NodeId {
        match self {
            Decl::Extern(node) => node.meta.id,
            Decl::Func(node) => node.meta.id,
        }
    }
}

impl Node for Stmt {
    fn pos(&self) -> &Pos {
        match self {
            Stmt::Return(node) => &node.meta.pos,
            Stmt::VarDecl(node) => &node.meta.pos,
            Stmt::VarAssign(node) => &node.meta.pos,
            Stmt::ExprStmt(expr) => expr.pos(),
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Stmt::Return(node) => &node.meta.end,
            Stmt::VarDecl(node) => &node.meta.end,
            Stmt::VarAssign(node) => &node.meta.end,
            Stmt::ExprStmt(expr) => expr.end(),
        }
    }

    fn id(&self) -> NodeId {
        match self {
            Stmt::Return(node) => node.meta.id,
            Stmt::VarDecl(node) => node.meta.id,
            Stmt::VarAssign(node) => node.meta.id,
            Stmt::ExprStmt(expr) => expr.id(),
        }
    }
}

impl Node for Expr {
    fn pos(&self) -> &Pos {
        match self {
            Expr::Literal(node) => &node.meta.pos,
            Expr::Call(node) => &node.meta.pos,
            Expr::Member(node) => &node.meta.pos,
            Expr::NamespaceMember(node) => &node.meta.pos,
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Expr::Literal(node) => &node.meta.end,
            Expr::Call(node) => &node.meta.end,
            Expr::Member(node) => &node.meta.end,
            Expr::NamespaceMember(node) => &node.meta.end,
        }
    }

    fn id(&self) -> NodeId {
        match self {
            Expr::Literal(node) => node.meta.id,
            Expr::Call(node) => node.meta.id,
            Expr::Member(node) => node.meta.id,
            Expr::NamespaceMember(node) => node.meta.id,
        }
    }
}

macro_rules! impl_typed_node_enum {
    ($enum:ty { $($variant:ident),* $(,)? }) => {
        impl<'a> TypedNode<'a> for $enum {
            fn type_id(&self) -> TypeId {
                match self {
                    $(Self::$variant(inner) => inner.type_id(),)*
                }
            }

            fn kind(&'a self) -> &'a TypeKind {
                match self {
                    $(Self::$variant(inner) => inner.kind(),)*
                }
            }
        }
    };
}

impl_typed_node_enum!(Decl { Func, Extern });
impl_typed_node_enum!(Stmt {
    Return,
    VarDecl,
    VarAssign,
    ExprStmt
});
impl_typed_node_enum!(Expr {
    Call,
    Literal,
    Member,
    NamespaceMember
});

macro_rules! impl_typed_node {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'a> TypedNode<'a> for $t {
                fn kind(&'a self) -> &'a TypeKind {
                    &self.ty.kind
                }

                fn type_id(&self) -> TypeId {
                    self.ty.id
                }
            }
        )*
    }
}

impl_typed_node!(
    ExternNode,
    FuncNode,
    LiteralNode,
    CallNode,
    ReturnNode,
    VarDeclNode,
    VarAssignNode,
    NamespaceMemberNode,
    MemberNode,
);
