use crate::{
    ast::{Ast, BlockNode, FuncNode, ReturnNode, Stmt, TypeNode, Visitable, Visitor},
    token::Token,
};

pub struct Printer {
    s: String,
    indent: usize,
}

impl Printer {
    /// Convert AST to printable format and print to stdout
    pub fn print(ast: &Ast) {
        println!("{}", Printer::to_string(ast));
    }

    /// Convert AST to printable format
    pub fn to_string(ast: &Ast) -> String {
        let mut s = Self {
            s: String::new(),
            indent: 0,
        };

        for node in &ast.nodes {
            node.accept(&mut s);
        }

        s.s.clone()
    }

    fn token(&mut self, token: &Token) {
        self.s.push_str(&format!("{}", token.kind));
    }
}

impl Visitor<()> for Printer {
    fn visit_literal(&mut self, node: &Token) {
        self.token(node);
    }

    fn visit_return(&mut self, node: &ReturnNode) {
        self.s.push_str("return");
        if let Some(expr) = &node.expr {
            self.s.push(' ');
            expr.accept(self);
        }
        self.s.push('\n');
    }

    fn visit_func(&mut self, node: &FuncNode) {
        self.s.push_str("func ");
        self.token(&node.name);
        self.s.push('(');

        if node.params.is_some() {
            for (i, param) in node.params.as_ref().unwrap().iter().enumerate() {
                if i > 0 {
                    self.s.push_str(", ");
                }
                self.token(&param.name);
                self.s.push(' ');
                param.typ.accept(self);
            }
        }

        self.s.push(')');
        self.s.push(' ');

        node.ret_type.as_ref().map(|t| {
            t.accept(self);
            self.s.push(' ');
        });

        Stmt::Block(node.body.clone()).accept(self);
    }

    fn visit_block(&mut self, node: &BlockNode) {
        self.s.push('{');
        self.s.push('\n');
        self.indent += 1;
        for stmt in &node.stmts {
            for _ in 0..self.indent {
                self.s.push_str("    ");
            }
            stmt.accept(self);
        }
        self.indent -= 1;
        for _ in 0..self.indent {
            self.s.push_str("    ");
        }
        self.s.push('}');
        self.s.push('\n');
        self.s.push('\n');
    }

    fn visit_type(&mut self, node: &super::TypeNode) -> () {
        match node {
            TypeNode::Primitive(tok) | TypeNode::Ident(tok) => self.visit_literal(tok),
        }
    }

    fn visit_package(&mut self, node: &Token) -> () {
        self.s.push_str(format!("package {}\n\n", node).as_str());
    }
}
