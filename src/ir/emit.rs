use std::mem;

use tracing::info;

use crate::{
    ast::{
        BlockNode, CallExpr, Decl, Expr, FuncNode, GroupExpr, Node, ReturnNode, TypeNode,
        Visitable, Visitor,
    },
    config::Config,
    error::{Error, ErrorSet, Res},
    ir::{ExternFuncInst, FuncInst, IRUnit, Ins, StringDataIns, SymTracker, Type, Value, ir},
    token::{Token, TokenKind},
    types::{self, Package, TypeContext, TypeId, TypeKind},
};

pub fn emit_ir(pkg: &Package, config: &Config) -> Res<IRUnit> {
    let emitter = Emitter::new(pkg, config);
    emitter.emit().map(|ins| IRUnit::new(ins))
}

struct Emitter<'a> {
    ctx: &'a TypeContext,
    nodes: &'a [Decl],
    sym: SymTracker,
    _config: &'a Config,

    ins: Vec<Vec<Ins>>,

    // Track if void functions have returned or not to add explicit return
    has_returned: bool,
    curstr: usize,
}

// TODO: dead code elimination (warning)

impl<'a> Emitter<'a> {
    fn new(pkg: &'a Package, config: &'a Config) -> Self {
        info!("package '{}' at {}", pkg.name, pkg.filepath);
        Self {
            _config: config,
            ctx: &pkg.ctx,
            nodes: &pkg.nodes,
            sym: SymTracker::new(),
            has_returned: false,
            ins: vec![Vec::new()],
            curstr: 0,
        }
    }

    fn emit(mut self) -> Res<Vec<Ins>> {
        let mut errs = ErrorSet::new();

        for decl in self.nodes {
            match decl.accept(&mut self) {
                Ok(_) => {}
                Err(err) => errs.add(err),
            }
        }

        if errs.len() == 0 {
            info!("success, {} instructions", self.ins.len());
            Ok(mem::take(&mut self.ins[0]))
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

    fn push_scope(&mut self) {
        self.ins.push(Vec::new());
    }

    fn pop_scope(&mut self) -> Vec<Ins> {
        self.ins.pop().expect("scope list is empty")
    }

    fn push(&mut self, ins: Ins) {
        self.ins.last_mut().expect("scope list is empty").push(ins);
    }

    fn next_string_name(&mut self) -> String {
        self.curstr += 1;
        format!("S{}", self.curstr)
    }

    /// Get the function signature as IR types. Returns a tuple of param types and return type.
    fn get_function_signature(&mut self, node: &dyn Node) -> Result<(Vec<Type>, Type), Error> {
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

        Ok((params, ret))
    }
}

impl<'a> Visitor<Result<Value, Error>> for Emitter<'a> {
    fn visit_func(&mut self, node: &FuncNode) -> Result<Value, Error> {
        self.sym.new_function_context();

        let name = node.name.to_string();
        let (params, ret) = self.get_function_signature(node)?;

        // Declare param indecies
        for p in &node.params {
            self.sym.set_param(p.name.to_string());
        }

        // Generate function body IR
        self.has_returned = false;
        self.push_scope();

        for stmt in &node.body.stmts {
            stmt.accept(self)?;
        }

        let mut body = self.pop_scope();

        // Add explicit void return for non-returing functions
        if !self.has_returned {
            body.push(Ins::Return(
                Type::Primitive(ir::Primitive::Void),
                Value::Void,
            ));
        }

        self.push(Ins::Func(FuncInst {
            name,
            public: node.public,
            params,
            ret,
            body,
        }));

        Ok(Value::Void)
    }

    fn visit_return(&mut self, node: &ReturnNode) -> Result<Value, Error> {
        let id = self.ctx.get_node(node);
        let ty = self.semtype_to_irtype(id);
        let val = node
            .expr
            .as_ref()
            .map_or(Ok(Value::Void), |expr| expr.accept(self))?;

        self.has_returned = true;
        self.push(Ins::Return(ty, val));

        Ok(Value::Void)
    }

    fn visit_literal(&mut self, token: &Token) -> Result<Value, Error> {
        Ok(match &token.kind {
            TokenKind::True => Value::Int(1),
            TokenKind::False => Value::Int(0),
            TokenKind::IntLit(n) => Value::Int(*n),
            TokenKind::FloatLit(n) => Value::Float(*n),
            TokenKind::CharLit(n) => Value::Int((*n).into()),
            TokenKind::IdentLit(name) => self.sym.get(name),
            TokenKind::StringLit(n) => {
                let name = self.next_string_name();

                self.push(Ins::StringData(StringDataIns {
                    name: name.to_owned(),
                    length: n.len(),
                    value: n.to_owned(),
                }));

                Value::Data(name.to_owned())
            }
            _ => panic!("unhandled token kind in evaluate: {:?}", token.kind),
        })
    }

    fn visit_call(&mut self, call: &CallExpr) -> Result<Value, Error> {
        let callee = match &*call.callee {
            Expr::Literal(t) => match &t.kind {
                TokenKind::IdentLit(name) => Value::Function(name.clone()),
                _ => panic!("unchecked invalid function call"),
            },
            e => e.accept(self)?,
        };

        let args = call
            .args
            .iter()
            .map(|arg| {
                let value = arg.accept(self)?;
                let ty = self.semtype_to_irtype(self.ctx.get_node(arg));
                Ok((ty, value))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let ty = self.semtype_to_irtype(self.ctx.get_node(call));
        let result = self.sym.next(); // declare after evaluating args to avoid incorrect id order

        self.push(Ins::Call(ir::CallIns {
            callee,
            ty,
            args,
            result,
        }));

        Ok(Value::Const(result))
    }

    fn visit_extern(&mut self, node: &crate::ast::FuncDeclNode) -> Result<Value, Error> {
        let name = node.name.to_string();
        let (params, ret) = self.get_function_signature(node)?;
        self.push(Ins::Extern(ExternFuncInst { name, params, ret }));
        Ok(Value::Void)
    }

    fn visit_group(&mut self, group: &GroupExpr) -> Result<Value, Error> {
        group.inner.accept(self)
    }

    fn visit_block(&mut self, _: &BlockNode) -> Result<Value, Error> {
        panic!("unused method")
    }

    fn visit_type(&mut self, _: &TypeNode) -> Result<Value, Error> {
        panic!("unused method")
    }

    fn visit_package(&mut self, _: &Token) -> Result<Value, Error> {
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
        types::PrimitiveType::String => ir::Primitive::Str,
    }
}
