use core::panic;
use std::{
    collections::HashMap,
    mem,
    sync::atomic::{AtomicUsize, Ordering},
};

use tracing::{debug, info};

use crate::{
    context::Context,
    error::{Diagnostics, Report, Res},
    ir::{
        self, Block, CallIns, ConstId, Data, DataIndex, Decl, ExternDecl, FuncDecl, IRType,
        IRTypeInterner, Ins, LValue, RValue, StoreIns, SymTracker, Unit,
    },
    module::{
        Module, ModuleId, ModuleKind, ModulePath, ModuleSourceFile, NamespaceList, Symbol,
        SymbolId, SymbolList, SymbolOrigin,
    },
    types::{
        self, Expr, FunctionType, LiteralKind, TypeId, TypeKind, TypedAst, TypedNode, Visitable,
        Visitor,
    },
};

// pub fn emit_ir(ctx: &Context, id: ModuleId) -> Res<Unit> {
//     // let module = ctx.modules.get(id);
//     // let ModuleKind::Source { files, .. } = &module.kind else {
//     //     panic!("attempt to emit non-source module");
//     // };

//     // let mut ins = Vec::new();

//     // for file in files {
//     //     let modulefile = ModuleFile {
//     //         modpath: &module.modpath,
//     //         nsl: &file.namespaces,
//     //         syms: &module.symbols,
//     //         nodes: &file.ast.decls,
//     //     };

//     //     let emitter = Emitter::new(ctx, modulefile);
//     //     let file_ins = emitter.emit()?;
//     //     ins.extend(file_ins);
//     // }

//     // Ok(Unit {
//     //     ins,
//     //     modpath: module.modpath.clone(),
//     // })

//     todo!()
// }

pub fn emit_ir(ctx: &Context, id: ModuleId) -> Res<Unit> {
    let module = ctx.modules.get(id);
    let emitter = ModuleEmitter::new(ctx, module);
    let unit = emitter.emit()?;
    Ok(unit)
}

struct DataInterner {
    data: Vec<Data>,
    string_map: HashMap<String, DataIndex>,
}

impl DataInterner {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            string_map: HashMap::new(),
        }
    }

    fn into_data(self) -> Vec<Data> {
        self.data
    }

    fn intern(&mut self, data: Data) -> DataIndex {
        let index = self.data.len();
        self.data.push(data);
        index
    }

    fn get_or_intern_string(&mut self, s: String) -> DataIndex {
        if let Some(index) = self.string_map.get(&s) {
            *index
        } else {
            let id = self.intern(Data::String(s.clone()));
            self.string_map.insert(s, id);
            id
        }
    }
}

struct ModuleEmitter<'a> {
    ctx: &'a Context,
    module: &'a Module,
    types: IRTypeInterner,
}

impl<'a> ModuleEmitter<'a> {
    fn new(ctx: &'a Context, module: &'a Module) -> Self {
        Self {
            ctx,
            module,
            types: IRTypeInterner::new(),
        }
    }

    fn emit(mut self) -> Res<Unit> {
        let ModuleKind::Source { files, .. } = &self.module.kind else {
            unreachable!();
        };

        let mut data = DataInterner::new();
        let mut decls = Vec::new();

        for id in &self.module.imports() {
            let mut diag = Diagnostics::new();

            match self.emit_extern(*id) {
                Ok(decl) => decls.push(decl),
                Err(report) => diag.add(report),
            }

            if !diag.is_empty() {
                return Err(diag);
            }
        }

        for file in files {
            let emitter = FileEmitter::new(
                self.ctx,
                &self.module.symbols,
                &mut self.types,
                file,
                &mut data,
            );
            let result = emitter.emit()?;
            decls.extend(result.decls);
        }

        Ok(Unit {
            name: self.module.modpath.to_underscore(),
            data: data.into_data(),
            types: self.types,
            decls,
        })
    }

    fn emit_extern(&mut self, id: SymbolId) -> Result<Decl, Report> {
        let symbol = self.ctx.symbols.get(id);
        let func = get_function_type(self.ctx, symbol.ty);

        Ok(Decl::Extern(ExternDecl {
            name: symbol.name.clone(),
            params: self.types.to_ir_type_list(self.ctx, &func.params),
            ret: self.types.to_ir_type_id(self.ctx, func.ret),
        }))
    }
}

pub struct FileEmitter<'a> {
    ctx: &'a Context,
    symbols: &'a SymbolList,
    types: &'a mut IRTypeInterner,
    nsl: &'a NamespaceList,
    ast: &'a TypedAst,
    data: &'a mut DataInterner,

    const_id: ConstId,
}

struct EmitResult {
    decls: Vec<Decl>,
}

impl<'a> FileEmitter<'a> {
    fn new(
        ctx: &'a Context,
        symbols: &'a SymbolList,
        types: &'a mut IRTypeInterner,
        file: &'a ModuleSourceFile,
        data: &'a mut DataInterner,
    ) -> Self {
        Self {
            ctx,
            symbols,
            data,
            types,
            nsl: &file.namespaces,
            ast: &file.ast,
            const_id: 0,
        }
    }

    fn emit(mut self) -> Res<EmitResult> {
        let mut diag = Diagnostics::new();
        let mut decls = Vec::new();

        for decl in &self.ast.decls {
            let res = match decl {
                types::Decl::Func(node) => self.emit_func(node),

                // extern symbols are declared at top using modules symbols list
                types::Decl::Extern(_) => continue,
            };

            match res {
                Ok(decl) => decls.push(decl),
                Err(report) => diag.add(report),
            }
        }

        Ok(EmitResult { decls })
    }

    fn next_id(&mut self) -> ConstId {
        let id = self.const_id;
        self.const_id += 1;
        id
    }

    fn emit_func(&mut self, node: &types::FuncNode) -> Result<Decl, Report> {
        let body = self.emit_block(&node.body)?;

        let func = get_function_type(self.ctx, node.ty.id);
        let params = self.types.to_ir_type_list(self.ctx, &func.params);
        let ret = self.types.to_ir_type_id(self.ctx, func.ret);

        Ok(Decl::Func(FuncDecl {
            public: node.public,
            name: node.name.clone(),
            body,
            params,
            ret,
            stacksize: 0,
        }))
    }

    fn emit_block(&mut self, nodes: &Vec<types::Stmt>) -> Result<Block, Report> {
        let mut ins = Vec::new();

        for node in nodes {
            match node {
                types::Stmt::Return(node) => self.emit_return(&mut ins, node)?,
                types::Stmt::ExprStmt(node) => {
                    let _ = self.expr_to_rval(&mut ins, node)?;
                }
                types::Stmt::VarDecl(node) => todo!(),
                types::Stmt::VarAssign(node) => todo!(),
            };
        }

        Ok(Block { ins })
    }

    fn emit_return(&mut self, ins: &mut Vec<Ins>, node: &types::ReturnNode) -> Result<(), Report> {
        let ty = self.types.to_ir_type_id(self.ctx, node.ty.id);
        let rval = match &node.expr {
            None => RValue::Void,
            Some(expr) => self.expr_to_rval(ins, &expr)?,
        };

        ins.push(Ins::Return(ty, rval));
        Ok(())
    }

    fn expr_to_rval(&mut self, ins: &mut Vec<Ins>, expr: &types::Expr) -> Result<RValue, Report> {
        match expr {
            Expr::Literal(node) => self.lit_to_rval(node),
            Expr::Call(node) => self.call_to_rval(ins, node),
            Expr::Member(node) => todo!(),
            Expr::NamespaceMember(node) => todo!(),
        }
    }

    fn lit_to_rval(&mut self, node: &types::LiteralNode) -> Result<RValue, Report> {
        Ok(match &node.kind {
            LiteralKind::Int(n) => RValue::Int(*n),
            LiteralKind::String(s) => RValue::Data(self.data.get_or_intern_string(s.to_owned())),
            LiteralKind::Ident(_) => todo!(),
            LiteralKind::Uint(_) => todo!(),
            LiteralKind::Float(_) => todo!(),
            LiteralKind::Bool(_) => todo!(),
            LiteralKind::Char(_) => todo!(),
        })
    }

    fn call_to_rval(
        &mut self,
        ins: &mut Vec<Ins>,
        node: &types::CallNode,
    ) -> Result<RValue, Report> {
        let args = node
            .args
            .iter()
            .map(|expr| {
                let ty = self.types.to_ir_type_id(self.ctx, expr.type_id());
                let rval = self.expr_to_rval(ins, expr)?;
                Ok((ty, rval))
            })
            .collect::<Result<Vec<_>, Report>>()?;

        let callee = match try_to_identifier(&node.callee) {
            Some(func_name) => RValue::Function(func_name.to_owned()),
            None => self.expr_to_rval(ins, &node.callee)?,
        };

        let result_id = self.next_id();

        ins.push(Ins::Call(CallIns {
            ty: self.types.to_ir_type_id(self.ctx, node.ty.id),
            result: LValue::Const(result_id),
            callee,
            args,
        }));

        Ok(RValue::Const(result_id))
    }
}

fn try_to_identifier(expr: &types::Expr) -> Option<&str> {
    if let types::Expr::Literal(lit) = expr {
        if let types::LiteralKind::Ident(name) = &lit.kind {
            return Some(name);
        };
    };
    None
}

fn get_function_type<'a>(ctx: &'a Context, id: TypeId) -> &'a FunctionType {
    let ty = ctx.types.lookup(id);
    let TypeKind::Function(func) = &ty.kind else {
        panic!("expected type to be function");
    };
    func
}

// static STRING_ID: AtomicUsize = AtomicUsize::new(0);

// fn next_id() -> usize {
//     STRING_ID.fetch_add(1, Ordering::Relaxed)
// }

// struct ModuleFile<'a> {
//     modpath: &'a ModulePath,
//     nsl: &'a NamespaceList,
//     nodes: &'a [Decl],
//     syms: &'a SymbolList,
// }

// struct Emitter<'a> {
//     ctx: &'a Context,
//     file: ModuleFile<'a>,

//     sym: SymTracker,
//     ins: Vec<Vec<Ins>>,

//     // Track if void functions have returned or not to add explicit return
//     has_returned: bool,
//     stack_size: usize, // Cumulative stack size from declarations
// }

// impl<'a> Emitter<'a> {
//     fn new(ctx: &'a Context, file: ModuleFile<'a>) -> Self {
//         Self {
//             ctx,
//             file,
//             sym: SymTracker::new(),
//             has_returned: false,
//             ins: vec![Vec::new()],
//             stack_size: 0,
//         }
//     }

//     fn emit(mut self) -> Res<Vec<Ins>> {
//         info!("Emitting IR for module: {}", self.file.modpath.path());
//         let mut diag = Diagnostics::new();

//         for decl in self.file.nodes {
//             match decl.accept(&mut self) {
//                 Ok(_) => {}
//                 Err(err) => diag.add(err),
//             }
//         }

//         if diag.num_errors() == 0 {
//             debug!("Success: {} instructions", self.ins.len());
//             Ok(mem::take(&mut self.ins[0]))
//         } else {
//             info!("Fail: finished with {} errors", diag.num_errors());
//             Err(diag)
//         }
//     }

//     fn node_to_ir_type(&self, node: &dyn TypedNode) -> IRType {
//         self.to_ir_type(node.type_id())
//     }

//     /// Convert semantic type to IR type, lowering to primitive or union type.
//     fn to_ir_type(&self, id: TypeId) -> IRType {
//         let id = self.ctx.types.deep_resolve(id);
//         let ty = self.ctx.types.lookup(id);

//         match &ty.kind {
//             TypeKind::Primitive(p) => IRType::Primitive(p.clone().into()),
//             TypeKind::Function(f) => IRType::Function(
//                 f.params.iter().map(|p| self.to_ir_type(*p)).collect(),
//                 Box::new(self.to_ir_type(f.ret)),
//             ),
//             _ => panic!("unhandled kind {:?}", ty.kind),
//         }
//     }

//     fn push_scope(&mut self) {
//         self.stack_size = 0;
//         self.ins.push(Vec::new());
//     }

//     fn pop_scope(&mut self) -> (Vec<Ins>, usize) {
//         (
//             self.ins.pop().expect("scope list is empty"),
//             self.stack_size,
//         )
//     }

//     fn push(&mut self, ins: Ins) {
//         self.ins.last_mut().expect("scope list is empty").push(ins);
//     }

//     fn next_string_name(&mut self) -> String {
//         format!("S{}", next_id())
//     }

//     fn mangle_symbol_name(&self, sym: &Symbol) -> String {
//         if self.ctx.config.no_mangle_names || sym.no_mangle || sym.is_extern() || sym.name == "main"
//         {
//             sym.name.clone()
//         } else {
//             let modpath = match &sym.origin {
//                 SymbolOrigin::Module { modpath, .. } => modpath,
//                 SymbolOrigin::Library(modpath) => modpath,
//                 _ => unreachable!(),
//             };
//             format!("_{}_{}", modpath.to_underscore(), sym.name)
//         }
//     }

//     fn get_symbol(&self, name: &str) -> &Symbol {
//         self.ctx
//             .symbols
//             .get(self.file.syms.get(name).expect("not a symbol").id)
//     }

//     fn get_namespace_symbol(&self, namespace: &str, name: &str) -> &Symbol {
//         self.ctx.symbols.get(
//             self.file
//                 .nsl
//                 .get(namespace)
//                 .expect("not a namespace")
//                 .get(name)
//                 .expect("not a symbol"),
//         )
//     }
// }

// impl<'a> Visitor<Result<RValue, Report>> for Emitter<'a> {
//     fn visit_func(&mut self, node: &types::FuncNode) -> Result<RValue, Report> {
//         let IRType::Function(params, ret) = self.node_to_ir_type(node) else {
//             panic!("expected func to be function type, was {:?}", &node.ty);
//         };

//         self.sym.new_function_context();

//         // Declare param indecies
//         for p in &node.params {
//             self.sym.set_param(p.clone());
//         }

//         // Generate function body IR
//         self.has_returned = false;
//         self.push_scope();

//         for stmt in &node.body {
//             stmt.accept(self)?;
//         }

//         let (mut body, mut stacksize) = self.pop_scope();

//         // Add param sizes to total stack size
//         for p in &params {
//             stacksize += p.size();
//         }

//         // Add explicit void return for non-returing functions
//         if !self.has_returned {
//             body.push(Ins::Return(
//                 IRType::Primitive(ir::Primitive::Void),
//                 RValue::Void,
//             ));
//         }

//         let sym = self.get_symbol(&node.name);

//         self.push(Ins::Func(FuncDecl {
//             name: self.mangle_symbol_name(sym),
//             public: node.public,
//             params,
//             ret: *ret,
//             body,
//             stacksize,
//         }));

//         Ok(RValue::Void)
//     }

//     fn visit_return(&mut self, node: &types::ReturnNode) -> Result<RValue, Report> {
//         let ty = self.node_to_ir_type(node);
//         let val = node
//             .expr
//             .as_ref()
//             .map_or(Ok(RValue::Void), |expr| expr.accept(self))?;

//         self.has_returned = true;
//         self.push(Ins::Return(ty, val));

//         Ok(RValue::Void)
//     }

//     fn visit_var_assign(&mut self, node: &types::VarAssignNode) -> Result<RValue, Report> {
//         let lval = match node.lval.accept(self)? {
//             RValue::Const(id) => LValue::Const(id),
//             RValue::Param(id) => LValue::Param(id),
//             _ => panic!("illegal lvalue"),
//         };

//         let value = node.rval.accept(self)?;
//         let ty = self.node_to_ir_type(&node.rval);

//         self.push(Ins::Assign(AssignIns { lval, ty, value }));
//         Ok(RValue::Void)
//     }

//     fn visit_var_decl(&mut self, node: &types::VarDeclNode) -> Result<RValue, Report> {
//         let value = node.value.accept(self)?;
//         let ty = self.node_to_ir_type(&node.value);
//         let id = self.sym.set(node.name.to_string());
//         self.stack_size += ty.size();
//         self.push(Ins::Store(StoreIns { id, ty, value }));
//         Ok(RValue::Void)
//     }

//     fn visit_literal(&mut self, node: &types::LiteralNode) -> Result<RValue, Report> {
//         Ok(match &node.kind {
//             LiteralKind::Ident(name) => self.sym.get(name),
//             LiteralKind::String(s) => {
//                 let name = self.next_string_name();
//                 self.push(Ins::StringData(StringDecl {
//                     name: name.to_owned(),
//                     length: s.len(),
//                     value: s.clone(),
//                 }));
//                 RValue::Data(name.to_owned())
//             }
//             LiteralKind::Int(n) => RValue::Int(*n),
//             LiteralKind::Uint(n) => RValue::Int(*n as i64),
//             LiteralKind::Float(f) => RValue::Float(*f),
//             LiteralKind::Bool(b) => RValue::Int(if *b { 1 } else { 0 }),
//             LiteralKind::Char(c) => RValue::Int((*c).into()),
//         })
//     }

//     fn visit_extern(&mut self, node: &types::ExternNode) -> Result<RValue, Report> {
//         let IRType::Function(params, ret) = self.node_to_ir_type(node) else {
//             panic!("expected func to be function type, was {:?}", &node.ty);
//         };

//         self.push(Ins::Extern(ExternDecl {
//             name: node.name.clone(),
//             ret: *ret,
//             params,
//         }));
//         Ok(RValue::Void)
//     }

//     fn visit_call(&mut self, node: &types::CallNode) -> Result<RValue, Report> {
//         let callee = match &*node.callee {
//             Expr::Literal(t) => match &t.kind {
//                 LiteralKind::Ident(name) => {
//                     // Get the function symbol and generate the correct link name
//                     let sym = self.get_symbol(name);
//                     RValue::Function(self.mangle_symbol_name(sym))
//                 }
//                 _ => panic!("unchecked invalid function call"),
//             },
//             e => e.accept(self)?,
//         };

//         let args = node
//             .args
//             .iter()
//             .map(|arg| {
//                 let value = arg.accept(self)?;
//                 let ty = self.node_to_ir_type(arg);
//                 Ok((ty, value))
//             })
//             .collect::<Result<Vec<_>, _>>()?;

//         let ty = self.node_to_ir_type(node);
//         let result = self.sym.next(); // declare after evaluating args to avoid incorrect id order

//         self.push(Ins::Call(ir::CallIns {
//             callee,
//             ty,
//             args,
//             result,
//         }));

//         Ok(RValue::Const(result))
//     }

//     fn visit_member(&mut self, _node: &types::MemberNode) -> Result<RValue, Report> {
//         todo!()
//     }

//     fn visit_namespace_member(
//         &mut self,
//         node: &types::NamespaceMemberNode,
//     ) -> Result<RValue, Report> {
//         let sym = self.get_namespace_symbol(&node.name, &node.field);
//         let linkname = self.mangle_symbol_name(sym);

//         // Declare as extern function
//         let f = self.node_to_ir_type(node);
//         let IRType::Function(params, ret) = f else {
//             panic!("not a function");
//         };

//         self.push(Ins::Extern(ExternDecl {
//             name: linkname.clone(),
//             params,
//             ret: *ret,
//         }));

//         Ok(RValue::Function(linkname))
//     }
// }
