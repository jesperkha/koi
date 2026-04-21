use crate::{
    ast::{Node, NodeId, Pos, TokenKind},
    types::{NO_TYPE, TypeId},
};

pub trait TypedNode<'a> {
    /// Get the unique TypeId for this node, only to be used within the same module.
    fn type_id(&self) -> TypeId;
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
    If(IfNode),
    While(WhileNode),
    For(ForNode),
    Break(BreakNode),
    Continue(ContinueNode),
}

pub enum Expr {
    Literal(LiteralNode),
    Call(CallNode),
    Member(MemberNode),
    NamespaceMember(NamespaceMemberNode),
    Binary(BinaryNode),
    Unary(UnaryNode),
}

impl Expr {
    /// Try to get the inner identifier string if this is a Ident kind.
    pub fn try_identifier(&self) -> Option<&str> {
        if let Expr::Literal(lit) = self
            && let LiteralKind::Ident(name) = &lit.kind
        {
            return Some(name);
        }
        None
    }
}

pub struct BreakNode {
    pub meta: NodeMeta,
}

pub struct ContinueNode {
    pub meta: NodeMeta,
}

pub struct BlockNode {
    pub stmts: Vec<Stmt>,
}

pub struct FuncNode {
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub name: String,
    pub params: Vec<String>,
    pub public: bool,
    pub body: BlockNode,
}

pub struct ExternNode {
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub name: String,
}

pub enum ElseBlock {
    ElseIf(Box<IfNode>),
    Else(Box<BlockNode>),
    None,
}

pub struct IfNode {
    pub meta: NodeMeta,
    pub expr: Expr,
    pub block: BlockNode,
    pub elseif: Box<ElseBlock>,
}

pub struct WhileNode {
    pub meta: NodeMeta,
    pub expr: Expr,
    pub block: BlockNode,
}

pub struct ForNode {
    pub meta: NodeMeta,
    pub initializer: Box<Stmt>,
    pub condition: Box<Expr>,
    pub increment: Box<Stmt>,
    pub block: BlockNode,
}

pub struct ReturnNode {
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub expr: Option<Expr>,
}

pub struct MemberNode {
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub expr: Box<Expr>,
    pub field: String,
}

pub struct NamespaceMemberNode {
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub name: String,
    pub field: String,
}

pub struct LiteralNode {
    pub ty: TypeId,
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
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub name: String,
    pub value: Expr,
}

pub struct VarAssignNode {
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub lval: Expr,
    pub rval: Expr,
}

pub struct CallNode {
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Mult,
    Divide,
    Modulo,

    Equal,
    NotEqual,

    Greater,
    GreaterEq,
    Less,
    LessEq,

    LogicAnd,
    LogicOr,
}

pub struct BinaryNode {
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub lhs: Box<Expr>,
    pub op: BinaryOp,
    pub rhs: Box<Expr>,
}

#[derive(Clone)]
pub enum UnaryOp {
    LogicNot,
    Minus,
}

pub struct UnaryNode {
    pub ty: TypeId,
    pub meta: NodeMeta,
    pub op: UnaryOp,
    pub rhs: Box<Expr>,
}

/// Implement ast::Node trait for a typed ast enum.
macro_rules! impl_node_for_enum {
    (
        $enum_name:ident {
            meta => [ $( $meta_variant:ident ),* $(,)? ],
            delegate => [ $( $delegate_variant:ident ),* $(,)? ]
        }
    ) => {
        impl Node for $enum_name {
            fn pos(&self) -> &Pos {
                match self {
                    $(
                        $enum_name::$meta_variant(node) => &node.meta.pos,
                    )*
                    $(
                        $enum_name::$delegate_variant(node) => node.pos(),
                    )*
                }
            }

            fn end(&self) -> &Pos {
                match self {
                    $(
                        $enum_name::$meta_variant(node) => &node.meta.end,
                    )*
                    $(
                        $enum_name::$delegate_variant(node) => node.end(),
                    )*
                }
            }

            fn id(&self) -> NodeId {
                match self {
                    $(
                        $enum_name::$meta_variant(node) => node.meta.id,
                    )*
                    $(
                        $enum_name::$delegate_variant(node) => node.id(),
                    )*
                }
            }
        }
    };
}

impl_node_for_enum!(Decl {
    meta => [
        Func,
        Extern,
    ],
    delegate => []
});

impl_node_for_enum!(Stmt {
    meta => [
        Return,
        VarDecl,
        VarAssign,
        If,
        While,
        For,
        Break,
        Continue,
    ],
    delegate => [
        ExprStmt,
    ]
});

impl_node_for_enum!(Expr {
    meta => [
        Literal,
        Call,
        Member,
        NamespaceMember,
        Binary,
        Unary,
    ],
    delegate => []
});

/// Implement types::TypedNode trait for the whole enum.
macro_rules! impl_typed_node_enum {
    ($enum:ty { $($variant:ident),* $(,)? }) => {
        impl<'a> TypedNode<'a> for $enum {
            fn type_id(&self) -> TypeId {
                match self {
                    $(Self::$variant(inner) => inner.type_id(),)*
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
    ExprStmt,
    If,
    While,
    Break,
    Continue,
    For,
});

impl_typed_node_enum!(Expr {
    Call,
    Literal,
    Member,
    NamespaceMember,
    Unary,
    Binary,
});

/// Implement types::TypedNode trait for variants with no type.
macro_rules! impl_no_type_node {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'a> TypedNode<'a> for $t {
                fn type_id(&self) -> TypeId {
                    NO_TYPE
                }
            }
        )*
    }
}

impl_no_type_node!(WhileNode, IfNode, ForNode, BreakNode, ContinueNode,);

/// Implement TypedNode for each enum variant.
macro_rules! impl_typed_node {
    ($($t:ty),* $(,)?) => {
        $(
            impl<'a> TypedNode<'a> for $t {
                fn type_id(&self) -> TypeId {
                    self.ty
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
    UnaryNode,
    BinaryNode,
);

impl From<TokenKind> for BinaryOp {
    fn from(value: TokenKind) -> Self {
        match value {
            TokenKind::Plus => Self::Plus,
            TokenKind::Minus => Self::Minus,
            TokenKind::Star => Self::Mult,
            TokenKind::Slash => Self::Divide,
            TokenKind::Percent => Self::Modulo,
            TokenKind::EqEq => Self::Equal,
            TokenKind::NotEq => Self::NotEqual,
            TokenKind::Greater => Self::Greater,
            TokenKind::Less => Self::Less,
            TokenKind::GreaterEq => Self::GreaterEq,
            TokenKind::LessEq => Self::LessEq,
            TokenKind::OrOr => Self::LogicOr,
            TokenKind::AndAnd => Self::LogicAnd,
            _ => unreachable!(),
        }
    }
}
