use crate::{
    ast::{Node, NodeId},
    token::{Pos, TokenKind},
    types::{Type, TypeContext, TypeId, TypeKind},
};

pub trait TypedNode<'a> {
    /// Get the TypeKind of this node.
    fn kind(&'a self) -> &'a TypeKind;
    /// Get the unique TypeId for this node, only to be used within the same package.
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
}

pub struct TypedAst {
    pub ctx: TypeContext,
    pub decls: Vec<Decl>,
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

pub struct LiteralNode {
    pub ty: Type,
    pub meta: NodeMeta,
    pub tok: TokenKind,
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

impl<'a> TypedNode<'a> for Decl {
    fn type_id(&self) -> TypeId {
        match self {
            Decl::Func(node) => node.ty.id,
            Decl::Extern(node) => node.ty.id,
        }
    }

    fn kind(&'a self) -> &'a TypeKind {
        match self {
            Decl::Func(node) => &node.ty.kind,
            Decl::Extern(node) => &node.ty.kind,
        }
    }
}

impl Visitable for Decl {
    fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        match self {
            Decl::Func(node) => v.visit_func(node),
            Decl::Extern(node) => v.visit_extern(node),
        }
    }
}

impl<'a> TypedNode<'a> for Stmt {
    fn type_id(&self) -> TypeId {
        match self {
            Stmt::Return(node) => node.ty.id,
            Stmt::VarDecl(node) => node.ty.id,
            Stmt::VarAssign(node) => node.ty.id,
            Stmt::ExprStmt(node) => node.type_id(),
        }
    }

    fn kind(&'a self) -> &'a TypeKind {
        match self {
            Stmt::Return(node) => &node.ty.kind,
            Stmt::VarAssign(node) => &node.ty.kind,
            Stmt::VarDecl(node) => &node.ty.kind,
            Stmt::ExprStmt(node) => node.kind(),
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

impl<'a> TypedNode<'a> for Expr {
    fn type_id(&self) -> TypeId {
        match self {
            Expr::Literal(node) => node.ty.id,
            Expr::Call(node) => node.ty.id,
        }
    }

    fn kind(&'a self) -> &'a TypeKind {
        match self {
            Expr::Literal(node) => &node.ty.kind,
            Expr::Call(node) => &node.ty.kind,
        }
    }
}

impl Visitable for Expr {
    fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Literal(node) => v.visit_literal(node),
            Expr::Call(node) => v.visit_call(node),
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
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Expr::Literal(node) => &node.meta.end,
            Expr::Call(node) => &node.meta.end,
        }
    }

    fn id(&self) -> NodeId {
        match self {
            Expr::Literal(node) => node.meta.id,
            Expr::Call(node) => node.meta.id,
        }
    }
}
