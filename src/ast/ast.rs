use crate::token::{Pos, Token};

pub type NodeId = usize;

#[derive(Debug)]
pub struct Ast {
    pub imports: Vec<ImportNode>,
    // Declarations are the only top level statements in koi. They contain
    // all other statements and expressions. Eg. a function has a block
    // statement, which consists of multiple ifs and calls.
    pub decls: Vec<Decl>,
}

impl Ast {
    /// Walks the AST and applies the visitor to each node.
    pub fn walk<R>(&self, visitor: &mut dyn Visitor<R>) {
        for node in &self.decls {
            node.accept(visitor);
        }
    }
}

/// A node is any part of the AST, including statements, expressions, and
/// declarations. Visitors can traverse these nodes to perform operations
/// like linting, analysis, or transformations.
pub trait Node {
    /// Position of first token in node segment.
    fn pos(&self) -> &Pos;
    /// Position of last token in node segment.
    fn end(&self) -> &Pos;
    /// Unique id of the node. Is the offset of the node pos, which is
    /// guaranteed unique for all nodes in the same file.
    fn id(&self) -> NodeId;
}

pub trait Visitable {
    /// Accept a visitor to inspect this node. Must call the appropriate
    /// visit method on the visitor for this node.
    fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R;
}

pub trait Visitor<R> {
    fn visit_func(&mut self, node: &FuncNode) -> R;
    fn visit_extern(&mut self, node: &FuncDeclNode) -> R;
    fn visit_block(&mut self, node: &BlockNode) -> R;
    fn visit_return(&mut self, node: &ReturnNode) -> R;
    fn visit_literal(&mut self, node: &Token) -> R;
    fn visit_type(&mut self, node: &TypeNode) -> R;
    fn visit_call(&mut self, node: &CallExpr) -> R;
    fn visit_group(&mut self, node: &GroupExpr) -> R;
    fn visit_var_decl(&mut self, node: &VarDeclNode) -> R;
    fn visit_var_assign(&mut self, node: &VarAssignNode) -> R;
    fn visit_import(&mut self, node: &ImportNode) -> R;
    fn visit_member(&mut self, node: &MemberNode) -> R;
}

/// Declarations are not considered statements for linting purposes.
/// Functions, structs, enums etc are all top level statements, and
/// therefore declarations. This does not include variable declarations,
/// but does include constant declarations.
#[derive(Debug)]
pub enum Decl {
    Func(FuncNode),
    Extern(FuncDeclNode),
    Import(ImportNode),
}

/// Statements are found inside blocks. They have side effects and do
/// not result in a value.
#[derive(Debug, Clone)]
pub enum Stmt {
    ExprStmt(Expr),
    Return(ReturnNode),
    Block(BlockNode),
    VarDecl(VarDeclNode),
    VarAssign(VarAssignNode),
}

/// Expressions are evaluated to produce a value. They can be used
/// in statements or as part of other expressions.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Token),
    Group(GroupExpr),
    Call(CallExpr),
    Member(MemberNode),
}

/// A TypeNode is the AST representation of a type, not the semantic meaning.
#[derive(Debug, Clone)]
pub enum TypeNode {
    Primitive(Token),
    Ident(Token),
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub lparen: Token,
    pub args: Vec<Expr>,
    pub rparen: Token,
}

#[derive(Debug, Clone)]
pub struct MemberNode {
    pub expr: Box<Expr>,
    pub dot: Token,
    pub field: Token,
}

#[derive(Debug, Clone)]
pub struct VarDeclNode {
    pub constant: bool,
    pub name: Token,
    pub symbol: Token,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct ImportNode {
    pub kw: Token,
    /// Names separated by period.
    pub names: Vec<Token>,
    /// Named items to import inside curly braces.
    pub imports: Vec<Token>,
    /// Alias name. Can only be present if len(imports) is 0.
    pub alias: Option<Token>,
    /// Final token in statement. This may differ depending
    /// on what type of import statement is used.
    pub end_tok: Token,
}

#[derive(Debug, Clone)]
pub struct VarAssignNode {
    pub lval: Expr,
    pub equal: Token,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct GroupExpr {
    pub lparen: Token,
    pub inner: Box<Expr>,
    pub rparen: Token,
}

#[derive(Debug, Clone)]
pub struct ReturnNode {
    pub kw: Token,
    pub expr: Option<Expr>,
}

#[derive(Debug)]
pub struct FuncDeclNode {
    pub public: bool,
    pub name: Token,
    pub lparen: Token,
    pub params: Vec<Field>,
    pub rparen: Token,
    pub ret_type: Option<TypeNode>,
}

#[derive(Debug)]
pub struct FuncNode {
    pub public: bool,
    pub name: Token,
    pub lparen: Token,
    pub params: Vec<Field>,
    pub rparen: Token,
    pub ret_type: Option<TypeNode>,
    pub body: BlockNode,
}

#[derive(Debug, Clone)]
pub struct BlockNode {
    pub lbrace: Token,
    pub stmts: Vec<Stmt>,
    pub rbrace: Token,
}

#[derive(Debug)]
pub struct Field {
    pub name: Token,
    pub typ: TypeNode,
}

impl Node for TypeNode {
    fn pos(&self) -> &Pos {
        match self {
            TypeNode::Primitive(token) | TypeNode::Ident(token) => &token.pos,
        }
    }

    fn end(&self) -> &Pos {
        match self {
            TypeNode::Primitive(token) | TypeNode::Ident(token) => &token.end_pos,
        }
    }

    fn id(&self) -> usize {
        match self {
            TypeNode::Primitive(token) | TypeNode::Ident(token) => token.id,
        }
    }
}

impl Visitable for TypeNode {
    fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        visitor.visit_type(self)
    }
}

impl Node for Decl {
    fn pos(&self) -> &Pos {
        match self {
            Decl::Func(node) => node.pos(),
            Decl::Extern(node) => node.pos(),
            Decl::Import(node) => node.pos(),
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Decl::Func(node) => node.end(),
            Decl::Extern(node) => node.end(),
            Decl::Import(node) => node.end(),
        }
    }

    fn id(&self) -> usize {
        match self {
            Decl::Func(node) => node.id(),
            Decl::Extern(node) => node.id(),
            Decl::Import(node) => node.id(),
        }
    }
}

impl Node for ImportNode {
    fn pos(&self) -> &Pos {
        &self.kw.pos
    }

    fn end(&self) -> &Pos {
        &self.end_tok.end_pos
    }

    fn id(&self) -> NodeId {
        self.kw.id
    }
}

impl Node for FuncNode {
    fn pos(&self) -> &Pos {
        &self.name.pos
    }

    fn end(&self) -> &Pos {
        &self.name.end_pos
    }

    fn id(&self) -> NodeId {
        self.name.id
    }
}

impl Node for FuncDeclNode {
    fn pos(&self) -> &Pos {
        &self.name.pos
    }

    fn end(&self) -> &Pos {
        &self.name.end_pos
    }

    fn id(&self) -> NodeId {
        self.name.id
    }
}

impl Visitable for Decl {
    fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Decl::Func(node) => visitor.visit_func(node),
            Decl::Extern(node) => visitor.visit_extern(node),
            Decl::Import(node) => visitor.visit_import(node),
        }
    }
}

impl Node for Stmt {
    fn pos(&self) -> &Pos {
        match self {
            Stmt::ExprStmt(node) => node.pos(),
            Stmt::Return(node) => node.pos(),
            Stmt::Block(node) => node.pos(),
            Stmt::VarDecl(node) => node.pos(),
            Stmt::VarAssign(node) => node.pos(),
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Stmt::ExprStmt(node) => node.end(),
            Stmt::Return(node) => node.end(),
            Stmt::Block(node) => node.end(),
            Stmt::VarDecl(node) => node.end(),
            Stmt::VarAssign(node) => node.end(),
        }
    }

    fn id(&self) -> usize {
        match self {
            Stmt::ExprStmt(node) => node.id(),
            Stmt::Return(node) => node.id(),
            Stmt::Block(node) => node.id(),
            Stmt::VarDecl(node) => node.id(),
            Stmt::VarAssign(node) => node.id(),
        }
    }
}

impl Node for VarDeclNode {
    fn pos(&self) -> &Pos {
        &self.name.pos
    }

    fn end(&self) -> &Pos {
        self.expr.end()
    }

    fn id(&self) -> NodeId {
        self.name.id
    }
}

impl Node for VarAssignNode {
    fn pos(&self) -> &Pos {
        self.lval.pos()
    }

    fn end(&self) -> &Pos {
        self.expr.end()
    }

    fn id(&self) -> NodeId {
        self.equal.id
    }
}

impl Node for ReturnNode {
    fn pos(&self) -> &Pos {
        &self.kw.pos
    }

    fn end(&self) -> &Pos {
        self.expr.as_ref().map(|e| e.end()).unwrap_or(&self.kw.pos)
    }

    fn id(&self) -> NodeId {
        self.kw.id
    }
}

impl Node for BlockNode {
    fn pos(&self) -> &Pos {
        &self.lbrace.pos
    }

    fn end(&self) -> &Pos {
        &self.rbrace.pos
    }

    fn id(&self) -> NodeId {
        self.lbrace.id
    }
}

impl Visitable for Stmt {
    fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Stmt::ExprStmt(node) => node.accept(visitor),
            Stmt::Return(node) => visitor.visit_return(node),
            Stmt::Block(node) => visitor.visit_block(node),
            Stmt::VarDecl(node) => visitor.visit_var_decl(node),
            Stmt::VarAssign(node) => visitor.visit_var_assign(node),
        }
    }
}

impl Node for Expr {
    fn pos(&self) -> &Pos {
        match self {
            Expr::Literal(token) => &token.pos,
            Expr::Call(call) => call.pos(),
            Expr::Group(grp) => &grp.lparen.pos,
            Expr::Member(node) => node.expr.pos(),
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Expr::Literal(token) => &token.end_pos,
            Expr::Call(call) => call.end(),
            Expr::Member(node) => &node.field.end_pos,
            Expr::Group(grp) => &grp.rparen.end_pos,
        }
    }

    fn id(&self) -> usize {
        match self {
            Expr::Literal(token) => token.id,
            Expr::Call(call) => call.id(),
            Expr::Group(grp) => grp.rparen.id,
            Expr::Member(node) => node.dot.id,
        }
    }
}

impl Node for CallExpr {
    fn pos(&self) -> &Pos {
        self.callee.pos()
    }

    fn end(&self) -> &Pos {
        &self.rparen.end_pos
    }

    fn id(&self) -> NodeId {
        self.lparen.id
    }
}

impl Node for MemberNode {
    fn pos(&self) -> &Pos {
        self.expr.pos()
    }

    fn end(&self) -> &Pos {
        &self.field.end_pos
    }

    fn id(&self) -> NodeId {
        self.dot.id
    }
}

impl Visitable for Expr {
    fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Expr::Literal(token) => visitor.visit_literal(token),
            Expr::Call(call) => visitor.visit_call(&call),
            Expr::Group(grp) => visitor.visit_group(&grp),
            Expr::Member(node) => visitor.visit_member(&node),
        }
    }
}
