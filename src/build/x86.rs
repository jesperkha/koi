use crate::{
    build::{Builder, TransUnit},
    config::Config,
    ir::{IRUnit, IRVisitor, Value},
};

pub struct X86Builder<'a> {
    _config: &'a Config,
    src: String,
    indent: usize,
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
}

impl<'a> Builder<'a> for X86Builder<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            _config: config,
            src: String::new(),
            indent: 0,
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

        for ins in &f.body {
            ins.accept(self);
        }

        self.pop();
    }

    fn visit_ret(&mut self, _ty: &crate::ir::Type, v: &crate::ir::Value) {
        match v {
            Value::Void => {}
            Value::Int(n) => self.writeln(&format!("mov rax, {}", n)),

            Value::Str(_) => todo!(),
            Value::Float(_) => todo!(),
            Value::Const(_) => todo!(),
            Value::Param(_) => todo!(),
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

    fn visit_package(&mut self, _: &str) -> () {}
}
