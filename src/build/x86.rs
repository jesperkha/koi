use std::collections::HashMap;

use crate::{
    build::{Builder, TransUnit},
    config::Config,
    ir::{ConstId, IRUnit, IRVisitor, Value},
};

pub struct X86Builder<'a> {
    _config: &'a Config,
    src: String,
    indent: usize,
    regmap: HashMap<ConstId, String>,
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
}

impl<'a> Builder<'a> for X86Builder<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            _config: config,
            src: String::new(),
            indent: 0,
            regmap: HashMap::new(),
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
    fn visit_func(&mut self, f: &crate::ir::FuncInst) {
        if f.public {
            self.writeln(&format!(".globl {}", f.name));
        }

        self.writeln(&format!("{}:", f.name));
        self.push();

        self.writeln("push rbp");
        self.writeln("mov rbp, rsp");

        // TODO: stack alignment and fetch param regs based on type
        let registers = ["edi", "esi", "edx", "ecx", "r8", "r9"];
        for i in 0..f.params.len() {
            self.writeln(&format!("mov [rsp-{}], {}", (i + 1) * 4, registers[i]));
        }

        for ins in &f.body {
            ins.accept(self);
        }

        self.pop();
    }

    fn visit_ret(&mut self, _ty: &crate::ir::Type, v: &crate::ir::Value) {
        match v {
            Value::Void => {}
            Value::Int(n) => self.writeln(&format!("mov eax, {}", n)),
            Value::Const(id) => self.writeln(&format!("mov eax, {}", self.get(*id))),
            Value::Param(id) => self.writeln(&format!("mov eax, [rsp-{}]", (id + 1) * 4)),
            Value::Str(_) => todo!(),
            Value::Float(_) => todo!(),
            Value::Function(_) => todo!(),
        };

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
        let registers = ["edi", "esi", "edx", "ecx", "r8", "r9"];

        match &c.callee {
            Value::Function(name) => {
                for (i, arg) in c.args.iter().enumerate() {
                    match arg {
                        Value::Int(n) => self.writeln(&format!("mov {}, {}", registers[i], n)),
                        Value::Const(id) => {
                            self.writeln(&format!("mov {}, {}", registers[i], self.get(*id)))
                        }
                        _ => panic!("invalid call argument"),
                    }
                }
                self.writeln(&format!("call {}", name));
                self.bind(c.result, "eax");
            }
            _ => panic!("invalid call callee"),
        }
    }
}
