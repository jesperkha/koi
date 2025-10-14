use tracing::info;

use crate::{
    ast::{BlockNode, Decl, Expr, FuncNode, ReturnNode, TypeNode, Visitable, Visitor},
    config::Config,
    error::{Error, ErrorSet, Res},
    ir::{FuncInst, IRUnit, Ins, SymTracker, Type, Value, ir},
    token::{Token, TokenKind},
    types::{self, Package, TypeContext, TypeId, TypeKind},
};

pub fn emit_ir(pkg: &Package, config: &Config) -> Res<IRUnit> {
    let emitter = Emitter::new(pkg, config);
    emitter.emit()
}

struct Emitter<'a> {
    ctx: &'a TypeContext,
    nodes: &'a [Decl],
    sym: SymTracker,
    config: &'a Config,

    // Track if void functions have returned or not to add explicit return
    has_returned: bool,
}

// TODO: dead code elimination (warning)

impl<'a> Emitter<'a> {
    fn new(pkg: &'a Package, config: &'a Config) -> Self {
        info!("package '{}' at {}", pkg.name, pkg.filepath);
        Self {
            config,
            ctx: &pkg.ctx,
            nodes: &pkg.nodes,
            sym: SymTracker::new(),
            has_returned: false,
        }
    }

    fn emit(mut self) -> Res<IRUnit> {
        let mut ins = Vec::new();
        let mut errs = ErrorSet::new();

        for decl in self.nodes {
            match decl.accept(&mut self) {
                Ok(i) => ins.push(i),
                Err(err) => errs.add(err),
            }
        }

        if errs.len() == 0 {
            info!("success, {} instructions", ins.len());
            Ok(IRUnit::new(ins))
        } else {
            info!("fail, finished with {} errors", errs.len());
            Err(errs)
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
                TokenKind::IdentLit(name) => self.sym.get(name),
                _ => panic!("unhandled token kind in evaluate: {:?}", token.kind),
            },
            Expr::Call(call_expr) => todo!(),
        }
    }
}

impl<'a> Visitor<Result<Ins, Error>> for Emitter<'a> {
    fn visit_func(&mut self, node: &FuncNode) -> Result<Ins, Error> {
        self.sym.new_function_context();

        let name = node.name.to_string();
        let func_type = self.ctx.lookup(self.ctx.get_node(node));

        let TypeKind::Function(ref param_ids, ret_id) = func_type.kind else {
            // Not implemented correctly if not function type
            panic!("function type was not TypeKind::Function")
        };

        // Collect return and param types
        let ret = self.semtype_to_irtype(ret_id);
        let params = param_ids
            .iter()
            .map(|ty| self.semtype_to_irtype(*ty))
            .collect();

        // Declare param indecies
        if let Some(params) = &node.params {
            for p in params {
                self.sym.set_param(p.name.to_string());
            }
        }

        // Generate function body IR
        let mut body = Vec::new();
        self.has_returned = false;

        for stmt in &node.body.stmts {
            body.push(stmt.accept(self)?);
        }

        // Add explicit void return for non-returing functions
        if !self.has_returned {
            body.push(Ins::Return(
                Type::Primitive(ir::Primitive::Void),
                Value::Void,
            ));
        }

        Ok(Ins::Func(FuncInst {
            name,
            public: node.public,
            params,
            ret,
            body,
        }))
    }

    fn visit_return(&mut self, node: &ReturnNode) -> Result<Ins, Error> {
        let id = self.ctx.get_node(node);
        let ty = self.semtype_to_irtype(id);
        let val = node
            .expr
            .as_ref()
            .map_or(Value::Void, |expr| self.evaluate(&expr));

        self.has_returned = true;
        Ok(Ins::Return(ty, val))
    }

    fn visit_literal(&mut self, _: &Token) -> Result<Ins, Error> {
        panic!("unused method")
    }

    fn visit_block(&mut self, _: &BlockNode) -> Result<Ins, Error> {
        panic!("unused method")
    }

    fn visit_type(&mut self, _: &TypeNode) -> Result<Ins, Error> {
        panic!("unused method")
    }

    fn visit_package(&mut self, node: &Token) -> Result<Ins, Error> {
        Ok(Ins::Package(node.to_string()))
    }

    fn visit_call(&mut self, node: &crate::ast::CallExpr) -> Result<Ins, Error> {
        todo!()
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
