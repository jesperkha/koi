use crate::token::{Pos, Token};

/// A node is any part of the AST, including statements, expressions, and
/// declarations. Visitors can traverse these nodes to perform operations
/// like linting, analysis, or transformations.
pub trait Node {
    /// Position of first token in node segment.
    fn pos(&self) -> &Pos;
    /// Position of last token in node segment.
    fn end(&self) -> &Pos;

    /// Accept a visitor to inspect this node. Must call the appropriate
    /// visit method on the visitor for this node.
    fn accept(&self, visitor: &mut dyn Visitor);
}

#[derive(Debug)]
pub struct Ast {
    // Declarations are the only top level statements in koi. They contain
    // all other statements and expressions. Eg. a function has a block
    // statement, which consists of multiple ifs and calls.
    pub nodes: Vec<Decl>,
}

impl Ast {
    pub fn new() -> Self {
        Ast { nodes: Vec::new() }
    }

    /// Walks the AST and applites the visitor to each node.
    pub fn walk(&mut self, visitor: &mut dyn Visitor) {
        for node in &self.nodes {
            node.accept(visitor);
        }
    }

    pub fn add_node(&mut self, node: Decl) {
        self.nodes.push(node);
    }
}

pub trait Visitor {
    fn visit_literal(&mut self, node: &Token);
    fn visit_return(&mut self, node: &ReturnNode);
    fn visit_func(&mut self, node: &FuncNode);
    fn visit_block(&mut self, node: &BlockNode);
}

/// Declarations are not considered statements for linting purposes.
/// Functions, structs, enums etc are all top level statements, and
/// therefore declarations. This does not include variable declarations,
/// but does include constant declarations.
#[derive(Debug)]
pub enum Decl {
    Func(FuncNode),
}

/// Statements are found inside blocks. They have side effects and do
/// not result in a value.
#[derive(Debug, Clone)]
pub enum Stmt {
    ExprStmt(Expr),
    Return(ReturnNode),
    Block(BlockNode),
}

/// Expressions are evaluated to produce a value. They can be used
/// in statements or as part of other expressions.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Token),
}

/// A TypeNode is the AST representation of a type, not the semantic meaning.
#[derive(Debug)]
pub enum TypeNode {
    Primitive(Token),
}

#[derive(Debug, Clone)]
pub struct ReturnNode {
    pub kw: Token,
    pub expr: Option<Expr>,
}

#[derive(Debug)]
pub struct FuncNode {
    pub public: bool,
    pub name: Token,
    pub lparen: Token,
    pub params: Option<Vec<Field>>,
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

impl Node for Decl {
    fn pos(&self) -> &Pos {
        match self {
            Decl::Func(node) => &node.name.pos,
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Decl::Func(node) => &node.body.rbrace.pos,
        }
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        match self {
            Decl::Func(node) => visitor.visit_func(node),
        }
    }
}

impl Node for Stmt {
    fn pos(&self) -> &Pos {
        match self {
            Stmt::ExprStmt(node) => node.pos(),
            Stmt::Return(node) => &node.kw.pos,
            Stmt::Block(node) => &node.lbrace.pos,
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Stmt::ExprStmt(node) => node.end(),
            Stmt::Return(node) => node.expr.as_ref().map(|e| e.end()).unwrap_or(&node.kw.pos),
            Stmt::Block(node) => &node.rbrace.pos,
        }
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        match self {
            Stmt::ExprStmt(node) => node.accept(visitor),
            Stmt::Return(node) => visitor.visit_return(node),
            Stmt::Block(node) => visitor.visit_block(node),
        }
    }
}

impl Node for Expr {
    fn pos(&self) -> &Pos {
        match self {
            Expr::Literal(token) => &token.pos,
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Expr::Literal(token) => &token.pos,
        }
    }

    fn accept(&self, visitor: &mut dyn Visitor) {
        match self {
            Expr::Literal(token) => visitor.visit_literal(token),
        }
    }
}
