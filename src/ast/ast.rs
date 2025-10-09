use crate::token::{Pos, Source, Token};

pub type NodeId = usize;

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
    fn visit_block(&mut self, node: &BlockNode) -> R;
    fn visit_return(&mut self, node: &ReturnNode) -> R;
    fn visit_literal(&mut self, node: &Token) -> R;
    fn visit_type(&mut self, node: &TypeNode) -> R;
    fn visit_package(&mut self, node: &Token) -> R;
}

#[derive(Debug)]
pub struct File {
    /// Package name token at beginning of file
    pub package: Option<Token>,
    /// Package name as string, or 'unnamed' if not specified (test files)
    pub pkgname: String,
    // Declarations are the only top level statements in koi. They contain
    // all other statements and expressions. Eg. a function has a block
    // statement, which consists of multiple ifs and calls.
    pub nodes: Vec<Decl>,

    pub src: Source,
}

impl File {
    pub fn new(src: Source) -> Self {
        File {
            nodes: Vec::new(),
            package: None,
            pkgname: "unnamed".to_string(),
            src,
        }
    }

    /// Walks the AST and applites the visitor to each node.
    pub fn walk<R>(&self, visitor: &mut dyn Visitor<R>) {
        for node in &self.nodes {
            node.accept(visitor);
        }
    }

    pub fn add_node(&mut self, node: Decl) {
        self.nodes.push(node);
    }

    pub fn set_package(&mut self, t: Token) {
        self.pkgname = t.to_string();
        self.package = Some(t);
    }
}

/// Declarations are not considered statements for linting purposes.
/// Functions, structs, enums etc are all top level statements, and
/// therefore declarations. This does not include variable declarations,
/// but does include constant declarations.
#[derive(Debug)]
pub enum Decl {
    Package(Token),
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
#[derive(Debug, Clone)]
pub enum TypeNode {
    Primitive(Token),
    Ident(Token),
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
            Decl::Package(name) => &name.pos,
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Decl::Func(node) => node.end(),
            Decl::Package(name) => &name.end_pos,
        }
    }

    fn id(&self) -> usize {
        match self {
            Decl::Func(node) => node.id(),
            Decl::Package(name) => name.id,
        }
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

impl Visitable for Decl {
    fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Decl::Func(node) => visitor.visit_func(node),
            Decl::Package(name) => visitor.visit_package(name),
        }
    }
}

impl Node for Stmt {
    fn pos(&self) -> &Pos {
        match self {
            Stmt::ExprStmt(node) => node.pos(),
            Stmt::Return(node) => node.pos(),
            Stmt::Block(node) => node.pos(),
        }
    }

    fn end(&self) -> &Pos {
        match self {
            Stmt::ExprStmt(node) => node.end(),
            Stmt::Return(node) => node.end(),
            Stmt::Block(node) => node.end(),
        }
    }

    fn id(&self) -> usize {
        match self {
            Stmt::ExprStmt(node) => node.id(),
            Stmt::Return(node) => node.id(),
            Stmt::Block(node) => node.id(),
        }
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
            Expr::Literal(token) => &token.end_pos,
        }
    }

    fn id(&self) -> usize {
        match self {
            Expr::Literal(token) => token.id,
        }
    }
}

impl Visitable for Expr {
    fn accept<R>(&self, visitor: &mut dyn Visitor<R>) -> R {
        match self {
            Expr::Literal(token) => visitor.visit_literal(token),
        }
    }
}
