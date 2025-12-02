use core::panic;
use std::mem;

use tracing::info;

use crate::{
    config::Config,
    error::{Error, ErrorSet, Res},
    ir::{
        AssignIns, ExternFuncInst, FuncInst, IRType, IRUnit, Ins, LValue, StoreIns, StringDataIns,
        SymTracker, Value, ir,
    },
    types::{
        self, Decl, Expr, LiteralKind, Package, TypeContext, TypeId, TypeKind, TypedNode,
        Visitable, Visitor,
    },
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

    stack_size: usize, // Cumulative stack size from declarations
}

impl<'a> Emitter<'a> {
    fn new(pkg: &'a Package, config: &'a Config) -> Self {
        info!("package '{}'", pkg.name());
        Self {
            _config: config,
            ctx: pkg.context(),
            nodes: pkg.nodes(),
            sym: SymTracker::new(),
            has_returned: false,
            ins: vec![Vec::new()],
            curstr: 0,
            stack_size: 0,
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

    fn node_to_ir_type(&self, node: &dyn TypedNode) -> IRType {
        self.to_ir_type(node.type_id())
    }

    /// Convert semantic type to IR type, lowering to primitive or union type.
    fn to_ir_type(&self, id: TypeId) -> IRType {
        let id = self.ctx.deep_resolve(id);
        let ty = self.ctx.lookup(id);

        match &ty.kind {
            TypeKind::Primitive(p) => IRType::Primitive(type_primitive_to_ir_primitive(&p)),
            TypeKind::Function(f) => IRType::Function(
                f.params.iter().map(|p| self.to_ir_type(*p)).collect(),
                Box::new(self.to_ir_type(f.ret)),
            ),
            _ => panic!("unhandled kind {:?}", ty.kind),
        }
    }

    fn push_scope(&mut self) {
        self.stack_size = 0;
        self.ins.push(Vec::new());
    }

    fn pop_scope(&mut self) -> (Vec<Ins>, usize) {
        (
            self.ins.pop().expect("scope list is empty"),
            self.stack_size,
        )
    }

    fn push(&mut self, ins: Ins) {
        self.ins.last_mut().expect("scope list is empty").push(ins);
    }

    fn next_string_name(&mut self) -> String {
        self.curstr += 1;
        format!("S{}", self.curstr)
    }
}

impl<'a> Visitor<Result<Value, Error>> for Emitter<'a> {
    fn visit_func(&mut self, node: &types::FuncNode) -> Result<Value, Error> {
        let IRType::Function(params, ret) = self.node_to_ir_type(node) else {
            panic!("expected func to be function type, was {:?}", &node.ty);
        };

        self.sym.new_function_context();

        // Declare param indecies
        for p in &node.params {
            self.sym.set_param(p.clone());
        }

        // Generate function body IR
        self.has_returned = false;
        self.push_scope();

        for stmt in &node.body {
            stmt.accept(self)?;
        }

        let (mut body, mut stacksize) = self.pop_scope();

        // Add param sizes to total stack size
        for p in &params {
            stacksize += p.size();
        }

        // Add explicit void return for non-returing functions
        if !self.has_returned {
            body.push(Ins::Return(
                IRType::Primitive(ir::Primitive::Void),
                Value::Void,
            ));
        }

        self.push(Ins::Func(FuncInst {
            name: node.name.clone(),
            public: node.public,
            params,
            ret: *ret,
            body,
            stacksize,
        }));

        Ok(Value::Void)
    }

    fn visit_return(&mut self, node: &types::ReturnNode) -> Result<Value, Error> {
        let ty = self.node_to_ir_type(node);
        let val = node
            .expr
            .as_ref()
            .map_or(Ok(Value::Void), |expr| expr.accept(self))?;

        self.has_returned = true;
        self.push(Ins::Return(ty, val));

        Ok(Value::Void)
    }

    fn visit_var_assign(&mut self, node: &types::VarAssignNode) -> Result<Value, Error> {
        let lval = match node.lval.accept(self)? {
            Value::Const(id) => LValue::Const(id),
            Value::Param(id) => LValue::Param(id),
            _ => panic!("illegal lvalue"),
        };

        let value = node.rval.accept(self)?;
        let ty = self.node_to_ir_type(&node.rval);

        self.push(Ins::Assign(AssignIns { lval, ty, value }));
        Ok(Value::Void)
    }

    fn visit_var_decl(&mut self, node: &types::VarDeclNode) -> Result<Value, Error> {
        let value = node.value.accept(self)?;
        let ty = self.node_to_ir_type(&node.value);
        let id = self.sym.set(node.name.to_string());
        self.stack_size += ty.size();
        self.push(Ins::Store(StoreIns { id, ty, value }));
        Ok(Value::Void)
    }

    fn visit_literal(&mut self, node: &types::LiteralNode) -> Result<Value, Error> {
        Ok(match &node.kind {
            LiteralKind::Ident(name) => self.sym.get(name),
            LiteralKind::String(s) => {
                let name = self.next_string_name();
                self.push(Ins::StringData(StringDataIns {
                    name: name.to_owned(),
                    length: s.len(),
                    value: s.clone(),
                }));
                Value::Data(name.to_owned())
            }
            LiteralKind::Int(n) => Value::Int(*n),
            LiteralKind::Uint(n) => Value::Int(*n as i64),
            LiteralKind::Float(f) => Value::Float(*f),
            LiteralKind::Bool(b) => Value::Int(if *b { 1 } else { 0 }),
            LiteralKind::Char(c) => Value::Int((*c).into()),
        })
    }

    fn visit_extern(&mut self, node: &types::ExternNode) -> Result<Value, Error> {
        let IRType::Function(params, ret) = self.node_to_ir_type(node) else {
            panic!("expected func to be function type, was {:?}", &node.ty);
        };

        self.push(Ins::Extern(ExternFuncInst {
            name: node.name.clone(),
            ret: *ret,
            params,
        }));
        Ok(Value::Void)
    }

    fn visit_call(&mut self, node: &types::CallNode) -> Result<Value, Error> {
        let callee = match &*node.callee {
            Expr::Literal(t) => match &t.kind {
                LiteralKind::Ident(name) => Value::Function(name.clone()),
                _ => panic!("unchecked invalid function call"),
            },
            e => e.accept(self)?,
        };

        let args = node
            .args
            .iter()
            .map(|arg| {
                let value = arg.accept(self)?;
                let ty = self.node_to_ir_type(arg);
                Ok((ty, value))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let ty = self.node_to_ir_type(node);
        let result = self.sym.next(); // declare after evaluating args to avoid incorrect id order

        self.push(Ins::Call(ir::CallIns {
            callee,
            ty,
            args,
            result,
        }));

        Ok(Value::Const(result))
    }

    fn visit_member(&mut self, node: &types::MemberNode) -> Result<Value, Error> {
        todo!()
    }

    fn visit_namespace_member(
        &mut self,
        node: &types::NamespaceMemberNode,
    ) -> Result<Value, Error> {
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
        types::PrimitiveType::String => ir::Primitive::Str,
    }
}
