use core::panic;
use std::{
    mem,
    sync::atomic::{AtomicUsize, Ordering},
};

use tracing::{debug, info};

// TODO: (test) string symbols (.S1 etc) are not duplicated across units

use crate::{
    context::Context,
    error::{Diagnostics, Report, Res},
    ir::{
        self, AssignIns, ExternFuncInst, FuncInst, IRType, Ins, LValue, StoreIns, StringDataIns,
        SymTracker, Unit, Value,
    },
    module::{ModuleId, ModuleKind, ModulePath, NamespaceList, Symbol, SymbolList, SymbolOrigin},
    types::{self, Decl, Expr, LiteralKind, TypeId, TypeKind, TypedNode, Visitable, Visitor},
};

pub fn emit_ir(ctx: &Context, id: ModuleId) -> Res<Unit> {
    let module = ctx.modules.get(id);
    let ModuleKind::Source(kind) = &module.kind else {
        panic!("attempt to emit non-source module");
    };

    let mut ins = Vec::new();

    for file in &kind.files {
        let modulefile = ModuleFile {
            modpath: &module.modpath,
            nsl: &file.namespaces,
            syms: &module.symbols,
            nodes: &file.ast.decls,
        };

        let emitter = Emitter::new(ctx, modulefile);
        let file_ins = emitter.emit()?;
        ins.extend(file_ins);
    }

    Ok(Unit {
        ins,
        modpath: module.modpath.clone(),
    })
}

static STRING_ID: AtomicUsize = AtomicUsize::new(0);

fn next_id() -> usize {
    STRING_ID.fetch_add(1, Ordering::Relaxed)
}

struct ModuleFile<'a> {
    modpath: &'a ModulePath,
    nsl: &'a NamespaceList,
    nodes: &'a [Decl],
    syms: &'a SymbolList,
}

struct Emitter<'a> {
    ctx: &'a Context,
    file: ModuleFile<'a>,

    sym: SymTracker,
    ins: Vec<Vec<Ins>>,

    // Track if void functions have returned or not to add explicit return
    has_returned: bool,
    stack_size: usize, // Cumulative stack size from declarations
}

impl<'a> Emitter<'a> {
    fn new(ctx: &'a Context, file: ModuleFile<'a>) -> Self {
        Self {
            ctx,
            file,
            sym: SymTracker::new(),
            has_returned: false,
            ins: vec![Vec::new()],
            stack_size: 0,
        }
    }

    fn emit(mut self) -> Res<Vec<Ins>> {
        info!("Emitting IR for module: {}", self.file.modpath.path());
        let mut diag = Diagnostics::new();

        for decl in self.file.nodes {
            match decl.accept(&mut self) {
                Ok(_) => {}
                Err(err) => diag.add(err),
            }
        }

        if diag.num_errors() == 0 {
            debug!("Success: {} instructions", self.ins.len());
            Ok(mem::take(&mut self.ins[0]))
        } else {
            info!("Fail: finished with {} errors", diag.num_errors());
            Err(diag)
        }
    }

    fn node_to_ir_type(&self, node: &dyn TypedNode) -> IRType {
        self.to_ir_type(node.type_id())
    }

    /// Convert semantic type to IR type, lowering to primitive or union type.
    fn to_ir_type(&self, id: TypeId) -> IRType {
        let id = self.ctx.types.deep_resolve(id);
        let ty = self.ctx.types.lookup(id);

        match &ty.kind {
            TypeKind::Primitive(p) => IRType::Primitive(p.clone().into()),
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
        format!("S{}", next_id())
    }

    fn mangle_symbol_name(&self, sym: &Symbol) -> String {
        if self.ctx.config.no_mangle_names || sym.no_mangle || sym.is_extern() || sym.name == "main"
        {
            sym.name.clone()
        } else {
            let modpath = match &sym.origin {
                SymbolOrigin::Module { modpath, .. } => modpath,
                SymbolOrigin::Library(modpath) => modpath,
                _ => unreachable!(),
            };
            format!("_{}_{}", modpath.to_underscore(), sym.name)
        }
    }

    fn get_symbol(&self, name: &str) -> &Symbol {
        self.ctx
            .symbols
            .get(self.file.syms.get(name).expect("not a symbol").id)
    }

    fn get_namespace_symbol(&self, namespace: &str, name: &str) -> &Symbol {
        self.ctx.symbols.get(
            self.file
                .nsl
                .get(namespace)
                .expect("not a namespace")
                .get(name)
                .expect("not a symbol"),
        )
    }
}

impl<'a> Visitor<Result<Value, Report>> for Emitter<'a> {
    fn visit_func(&mut self, node: &types::FuncNode) -> Result<Value, Report> {
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

        let sym = self.get_symbol(&node.name);

        self.push(Ins::Func(FuncInst {
            name: self.mangle_symbol_name(sym),
            public: node.public,
            params,
            ret: *ret,
            body,
            stacksize,
        }));

        Ok(Value::Void)
    }

    fn visit_return(&mut self, node: &types::ReturnNode) -> Result<Value, Report> {
        let ty = self.node_to_ir_type(node);
        let val = node
            .expr
            .as_ref()
            .map_or(Ok(Value::Void), |expr| expr.accept(self))?;

        self.has_returned = true;
        self.push(Ins::Return(ty, val));

        Ok(Value::Void)
    }

    fn visit_var_assign(&mut self, node: &types::VarAssignNode) -> Result<Value, Report> {
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

    fn visit_var_decl(&mut self, node: &types::VarDeclNode) -> Result<Value, Report> {
        let value = node.value.accept(self)?;
        let ty = self.node_to_ir_type(&node.value);
        let id = self.sym.set(node.name.to_string());
        self.stack_size += ty.size();
        self.push(Ins::Store(StoreIns { id, ty, value }));
        Ok(Value::Void)
    }

    fn visit_literal(&mut self, node: &types::LiteralNode) -> Result<Value, Report> {
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

    fn visit_extern(&mut self, node: &types::ExternNode) -> Result<Value, Report> {
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

    fn visit_call(&mut self, node: &types::CallNode) -> Result<Value, Report> {
        let callee = match &*node.callee {
            Expr::Literal(t) => match &t.kind {
                LiteralKind::Ident(name) => {
                    // Get the function symbol and generate the correct link name
                    let sym = self.get_symbol(name);
                    Value::Function(self.mangle_symbol_name(sym))
                }
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

    fn visit_member(&mut self, _node: &types::MemberNode) -> Result<Value, Report> {
        todo!()
    }

    fn visit_namespace_member(
        &mut self,
        node: &types::NamespaceMemberNode,
    ) -> Result<Value, Report> {
        let sym = self.get_namespace_symbol(&node.name, &node.field);
        let linkname = self.mangle_symbol_name(sym);

        // Declare as extern function
        let f = self.node_to_ir_type(node);
        let IRType::Function(params, ret) = f else {
            panic!("not a function");
        };

        self.push(Ins::Extern(ExternFuncInst {
            name: linkname.clone(),
            params,
            ret: *ret,
        }));

        Ok(Value::Function(linkname))
    }
}
