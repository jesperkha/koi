use crate::{
    ast::{Ast, BlockNode, Expr, FuncNode, ReturnNode, TypeNode, Visitable, Visitor},
    error::ErrorSet,
    ir::{FuncInst, Ins, Type, Value, ir},
    token::{Token, TokenKind},
    types::{self, TypeContext, TypeId, TypeKind},
};

pub struct IR<'a> {
    ctx: &'a TypeContext,
    errs: ErrorSet,
    ins: Vec<Ins>,

    // Track if void functions have returned or not to add explicit return
    has_returned: bool,
}

pub type IRResult = Result<Vec<Ins>, ErrorSet>;

impl<'a> IR<'a> {
    pub fn emit(ast: &'a Ast, ctx: &'a TypeContext) -> IRResult {
        let mut s = Self {
            ctx,
            errs: ErrorSet::new(),
            ins: Vec::new(),
            has_returned: false,
        };

        ast.walk(&mut s);
        if s.errs.size() == 0 {
            Ok(s.ins)
        } else {
            Err(s.errs)
        }
    }

    /// Convert semantic type to IR type, lowering to primitive or union type.
    fn semtype_to_irtype(&self, id: TypeId) -> Type {
        let id = self.ctx.deep_resolve(id);
        let ty = self.ctx.lookup(id);

        match &ty.kind {
            TypeKind::Primitive(p) => Type::Primitive(type_primitive_to_ir_primitive(&p)),
            _ => panic!("unhandled kind {:?}", ty.kind),
        }
    }

    fn evaluate(&self, expr: &Expr) -> Value {
        match expr {
            Expr::Literal(token) => match &token.kind {
                TokenKind::True => Value::Int(1),
                TokenKind::False => Value::Int(0),
                TokenKind::IntLit(n) => Value::Int(*n),
                TokenKind::FloatLit(n) => Value::Float(*n),
                TokenKind::StringLit(n) => Value::Str(n.clone()),
                TokenKind::CharLit(n) => Value::Int((*n).into()),
                _ => panic!("unhandled token kind in evaluate: {:?}", token.kind),
            },
        }
    }
}

impl<'a> Visitor<()> for IR<'a> {
    fn visit_func(&mut self, node: &FuncNode) {
        let name = node.name.to_string();
        let func_type = self.ctx.lookup(self.ctx.get_node(node));

        let TypeKind::Function(ref param_ids, ret_id) = func_type.kind else {
            // Not implemented correctly if not function type
            panic!("function type was not TypeKind::Function")
        };

        let ret = self.semtype_to_irtype(ret_id);
        let params = param_ids
            .iter()
            .map(|ty| self.semtype_to_irtype(*ty))
            .collect();

        let func = Ins::Func(FuncInst { name, params, ret });
        self.ins.push(func);

        self.has_returned = false;
        self.visit_block(&node.body);

        // Add explicit void return for non-returing functions
        if !self.has_returned {
            self.ins.push(Ins::Return(
                Type::Primitive(ir::Primitive::Void),
                Value::Void,
            ));
        }
    }

    fn visit_block(&mut self, node: &BlockNode) {
        for stmt in &node.stmts {
            stmt.accept(self);
        }
    }

    fn visit_return(&mut self, node: &ReturnNode) {
        let id = self.ctx.get_node(node);
        let ty = self.semtype_to_irtype(id);
        let val = node
            .expr
            .as_ref()
            .map_or(Value::Void, |expr| self.evaluate(&expr));

        let ret = Ins::Return(ty, val);
        self.ins.push(ret);
        self.has_returned = true;
    }

    fn visit_literal(&mut self, _: &Token) {
        panic!("unused method")
    }

    fn visit_type(&mut self, _: &TypeNode) {
        panic!("unused method")
    }
}

fn type_primitive_to_ir_primitive(p: &types::PrimitiveType) -> ir::Primitive {
    match p {
        types::PrimitiveType::Void => ir::Primitive::Void,
        types::PrimitiveType::I8 => ir::Primitive::I8,
        types::PrimitiveType::I16 => ir::Primitive::I16,
        types::PrimitiveType::I32 => ir::Primitive::I32,
        types::PrimitiveType::I64 => ir::Primitive::I64,
        types::PrimitiveType::Byte | types::PrimitiveType::Bool | types::PrimitiveType::U8 => {
            ir::Primitive::U8
        }
        types::PrimitiveType::U16 => ir::Primitive::U16,
        types::PrimitiveType::U32 => ir::Primitive::U32,
        types::PrimitiveType::U64 => ir::Primitive::U64,
        types::PrimitiveType::F32 => ir::Primitive::F32,
        types::PrimitiveType::F64 => ir::Primitive::F64,
    }
}
