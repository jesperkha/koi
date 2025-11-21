use crate::{
    token::TokenKind,
    types::{Type, TypeContext, TypeId, TypeKind},
};

pub trait Node<'a> {
    /// Get the TypeKind of this node.
    fn kind(&'a self) -> &'a TypeKind;
    /// Get the unique TypeId for this node, only to be used within the same package.
    fn id(&self) -> TypeId;
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
}

pub struct TypedAst {
    pub package_name: String,
    pub ctx: TypeContext,
    pub decls: Vec<Decl>,
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
}

pub struct FuncNode {
    pub ty: Type,
    pub name: String,
    pub public: bool,
    pub body: Vec<Stmt>,
}

pub struct ExternNode {
    pub ty: Type,
    pub name: String,
}

pub struct ReturnNode {
    pub ty: Type,
    pub expr: Option<Expr>,
}

pub struct LiteralNode {
    pub ty: Type,
    pub tok: TokenKind,
}

pub struct VarDeclNode {
    pub ty: Type,
    pub name: String,
    pub value: Expr,
}

pub struct VarAssignNode {
    pub ty: Type,
    pub lval: Expr,
    pub rval: Expr,
}

impl<'a> Node<'a> for Decl {
    fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        match self {
            Decl::Func(node) => v.visit_func(node),
            Decl::Extern(node) => v.visit_extern(node),
        }
    }

    fn id(&self) -> TypeId {
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

impl<'a> Node<'a> for Stmt {
    fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        match self {
            Stmt::Return(node) => v.visit_return(node),
            Stmt::VarDecl(node) => v.visit_var_decl(node),
            Stmt::VarAssign(node) => v.visit_var_assign(node),
            Stmt::ExprStmt(node) => node.accept(v),
        }
    }

    fn id(&self) -> TypeId {
        match self {
            Stmt::Return(node) => node.ty.id,
            Stmt::VarDecl(node) => node.ty.id,
            Stmt::VarAssign(node) => node.ty.id,
            Stmt::ExprStmt(node) => node.id(),
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

impl<'a> Node<'a> for Expr {
    fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Literal(node) => v.visit_literal(node),
        }
    }

    fn id(&self) -> TypeId {
        match self {
            Expr::Literal(node) => node.ty.id,
        }
    }

    fn kind(&'a self) -> &'a TypeKind {
        match self {
            Expr::Literal(node) => &node.ty.kind,
        }
    }
}
