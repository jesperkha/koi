use std::collections::HashMap;

use crate::{
    build::{Builder, RegAllocator, TransUnit},
    config::Config,
    ir::{ConstId, IRUnit, IRVisitor, Value},
};

pub struct X86Builder<'a> {
    _config: &'a Config,
    src: String,
    indent: usize,
    regmap: HashMap<ConstId, String>,
    parammap: HashMap<ConstId, String>,
    alloc: RegAllocator,
    stacksize: usize,
}

impl<'a> X86Builder<'a> {
    fn write(&mut self, s: &str) {
        self.src
            .push_str(&format!("{}{}", "    ".repeat(self.indent), s));
    }

    fn writeln(&mut self, s: &str) {
        self.write(&format!("{}\n", s));
    }

    fn push(&mut self) {
        self.indent += 1
    }

    fn pop(&mut self) {
        self.indent -= 1;
    }

    fn bind(&mut self, id: ConstId, reg: &str) {
        self.regmap.insert(id, reg.to_string());
    }

    fn get(&self, id: ConstId) -> &str {
        self.regmap.get(&id).expect("unknown const id")
    }

    fn bind_param(&mut self, id: ConstId, location: &str) {
        self.parammap.insert(id, location.to_string());
    }

    fn get_param(&self, id: ConstId) -> &str {
        self.parammap.get(&id).expect("unknown const id")
    }

    fn value(&self, v: &Value) -> String {
        match v {
            Value::Void => panic!("cannot get value of void type"),
            Value::Int(n) => n.to_string(),
            Value::Const(id) => self.get(*id).to_string(),
            Value::Param(id) => self.get_param(*id).to_string(),

            Value::Str(_) => todo!(),
            Value::Float(_) => todo!(),
            Value::Function(_) => todo!(),
        }
    }

    fn mov(&mut self, dest: &str, value: &str) {
        self.writeln(&format!("mov {}, {}", dest, value));
    }

    /// Increases stack size and returns location for requested size.
    fn stack_alloc(&mut self, size: usize) -> String {
        self.stacksize += size.max(4);
        let directive = match size {
            1 => "BYTE",
            2 => "WORD",
            4 => "DWORD",
            8 => "QWORD",
            _ => panic!("illegal size: {}", size),
        };
        format!("{} [rbp-{}]", directive, self.stacksize)
    }
}

impl<'a> Builder<'a> for X86Builder<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            _config: config,
            src: String::new(),
            indent: 0,
            regmap: HashMap::new(),
            parammap: HashMap::new(),
            alloc: RegAllocator::new(),
            stacksize: 0,
        }
    }

    fn assemble(mut self, unit: IRUnit) -> Result<TransUnit, String> {
        self.writeln(".intel_syntax noprefix");
        self.writeln(".section .data");
        self.writeln(".section .text\n");

        for ins in &unit.ins {
            ins.accept(&mut self);
        }

        Ok(TransUnit { source: self.src })
    }
}

impl<'a> IRVisitor<()> for X86Builder<'a> {
    // TODO: sub rsp with aligned stack size
    fn visit_func(&mut self, f: &crate::ir::FuncInst) {
        if f.public {
            self.writeln(&format!(".globl {}", f.name));
        }

        self.writeln(&format!("{}:", f.name));
        self.push();

        self.writeln("push rbp");
        self.writeln("mov rbp, rsp");

        self.alloc.reset_params();
        for (i, ty) in f.params.iter().enumerate() {
            let dest = self.stack_alloc(ty.size());
            let reg = self.alloc.next_param_reg(ty);

            self.bind_param(i, &dest);
            self.mov(&dest, &reg);
        }

        for ins in &f.body {
            ins.accept(self);
        }

        self.pop();
    }

    fn visit_ret(&mut self, ty: &crate::ir::Type, v: &crate::ir::Value) {
        self.mov(&self.alloc.return_reg(ty), &self.value(v));
        self.writeln("leave");
        self.writeln("ret\n");
    }

    fn visit_store(
        &mut self,
        _id: crate::ir::ConstId,
        _ty: &crate::ir::Type,
        _v: &crate::ir::Value,
    ) {
        todo!()
    }

    fn visit_call(&mut self, c: &crate::ir::CallIns) -> () {
        self.alloc.reset_params();
        match &c.callee {
            Value::Function(name) => {
                for arg in &c.args {
                    let dest = self.alloc.next_param_reg(&arg.0);
                    let value = self.value(&arg.1);
                    self.mov(&dest, &value);
                }
                self.writeln(&format!("call {}", name));
                self.bind(c.result, &self.alloc.return_reg(&c.ty));
            }
            _ => panic!("invalid call callee"),
        }
    }
}
