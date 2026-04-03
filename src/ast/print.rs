use crate::ast::{
    Ast, BlockNode, ElseBlock, FuncNode, ReturnNode, Stmt, Token, TypeNode, Visitable, Visitor,
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

        for node in &ast.imports {
            s.visit_import(node);
        }

        for node in &ast.decls {
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
    }

    fn visit_func(&mut self, node: &FuncNode) {
        self.s.push_str("func ");
        self.token(&node.name);
        self.s.push('(');

        for (i, param) in node.params.iter().enumerate() {
            if i > 0 {
                self.s.push_str(", ");
            }
            self.token(&param.name);
            self.s.push(' ');
            param.typ.accept(self);
        }

        self.s.push(')');
        self.s.push(' ');

        if let Some(t) = node.ret_type.as_ref() {
            t.accept(self);
            self.s.push(' ');
        }

        Stmt::Block(node.body.clone()).accept(self);
        self.s += "\n";
        self.s += "\n";
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
            self.s.push('\n');
        }
        self.indent -= 1;
        for _ in 0..self.indent {
            self.s.push_str("    ");
        }
        self.s.push('}');
    }

    fn visit_type(&mut self, node: &super::TypeNode) {
        match node {
            TypeNode::Primitive(tok) | TypeNode::Ident(tok) => self.visit_literal(tok),
        }
    }

    fn visit_call(&mut self, node: &super::CallExpr) {
        node.callee.accept(self);
        self.s.push('(');
        for (i, arg) in node.args.iter().enumerate() {
            arg.accept(self);
            if i != node.args.len() - 1 {
                self.s.push_str(", ");
            }
        }
        self.s.push(')');
    }

    fn visit_group(&mut self, node: &super::GroupExpr) {
        self.s.push('(');
        node.inner.accept(self);
        self.s.push(')');
    }

    fn visit_extern(&mut self, node: &super::FuncDeclNode) {
        self.s.push_str("extern func ");
        self.token(&node.name);
        self.s.push('(');

        for (i, param) in node.params.iter().enumerate() {
            if i > 0 {
                self.s.push_str(", ");
            }
            self.token(&param.name);
            self.s.push(' ');
            param.typ.accept(self);
        }

        self.s.push(')');
        self.s.push(' ');

        if let Some(t) = node.ret_type.as_ref() {
            t.accept(self);
            self.s.push(' ');
        }

        self.s.push('\n');
        self.s.push('\n');
    }

    fn visit_var_decl(&mut self, node: &super::VarDeclNode) {
        self.s.push_str(&format!("{} {} ", node.name, node.symbol));
        node.expr.accept(self);
    }

    fn visit_var_assign(&mut self, node: &super::VarAssignNode) {
        node.lval.accept(self);
        self.s.push_str(" = ");
        node.expr.accept(self);
    }

    fn visit_import(&mut self, node: &super::ImportNode) {
        self.s.push_str(&format!(
            "import {} {}\n\n",
            node.names
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<_>>()
                .join("."),
            if let Some(alias) = &node.alias {
                format!("as {}", alias)
            } else if !node.imports.is_empty() {
                format!(
                    "{{\n    {}\n}}",
                    node.imports
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(",\n    ")
                )
            } else {
                "".to_string()
            }
        ));
    }

    fn visit_member(&mut self, node: &super::MemberNode) {
        node.expr.accept(self);
        self.s.push('.');
        self.s.push_str(&node.field.to_string());
    }

    fn visit_binary(&mut self, node: &super::BinaryExpr) {
        node.lhs.accept(self);
        self.s += " ";
        self.visit_literal(&node.op);
        self.s += " ";
        node.rhs.accept(self);
    }

    fn visit_unary(&mut self, node: &super::UnaryExpr) {
        self.visit_literal(&node.op);
        node.rhs.accept(self);
    }

    fn visit_if(&mut self, node: &super::IfNode) -> () {
        self.s += "if ";
        node.expr.accept(self);
        self.s += " ";
        self.visit_block(&node.block);
        if let Some(elseif) = &node.elseif {
            self.s += " else ";
            match elseif.as_ref() {
                ElseBlock::ElseIf(if_node) => self.visit_if(if_node),
                ElseBlock::Else(block) => self.visit_block(block),
            }
        }
    }
}
