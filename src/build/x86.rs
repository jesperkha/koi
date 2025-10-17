use std::collections::HashMap;

use crate::{
    build::{Builder, RegAllocator, TransUnit},
    config::Config,
    ir::{ConstId, IRUnit, IRVisitor, Type, Value},
};

pub struct X86Builder<'a> {
    _config: &'a Config,
    regmap: HashMap<ConstId, String>,
    parammap: HashMap<ConstId, String>,
    alloc: RegAllocator,
    stacksize: usize,

    head: Writer,
    text: Writer,
    data: Writer,
}

struct Writer {
    indent: usize,
    content: String,
}

impl Writer {
    fn new() -> Self {
        Self {
            indent: 0,
            content: String::new(),
        }
    }

    fn write(&mut self, s: &str) {
        self.content
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

    fn append(&mut self, other: &Writer) {
        self.content.push_str(&other.content);
        self.content.push_str("\n");
    }
}

impl<'a> X86Builder<'a> {
    fn push(&mut self) {
        self.text.push();
    }

    fn pop(&mut self) {
        self.text.pop();
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
            Value::Float(_) => todo!(),
            Value::Function(_) => todo!(),
            Value::Data(name) => format!("[rip + .{}]", name),
        }
    }

    fn mov_str(&mut self, dest: &str, value: &str, _ty: &Type) {
        self.text.writeln(&format!("mov {}, {}", dest, value));
    }

    fn mov(&mut self, dest: &str, value: &Value, _ty: &Type) {
        let kw = match value {
            Value::Data(_) => "lea",
            _ => "mov",
        };
        self.text
            .writeln(&format!("{} {}, {}", kw, dest, self.value(value)));
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
        format!("{} PTR [rbp-{}]", directive, self.stacksize)
    }
}

impl<'a> Builder<'a> for X86Builder<'a> {
    fn new(config: &'a Config) -> Self {
        Self {
            _config: config,

            head: Writer::new(),
            text: Writer::new(),
            data: Writer::new(),

            regmap: HashMap::new(),
            parammap: HashMap::new(),
            alloc: RegAllocator::new(),
            stacksize: 0,
        }
    }

    fn assemble(mut self, unit: IRUnit) -> Result<TransUnit, String> {
        self.head.writeln(".intel_syntax noprefix");
        self.data.push();

        for ins in &unit.ins {
            ins.accept(&mut self);
        }

        let mut src = Writer::new();
        src.append(&self.head);

        src.writeln(".section .data\n");
        src.append(&self.data);

        src.writeln(".section .text\n");
        src.append(&self.text);

        Ok(TransUnit {
            source: src.content,
        })
    }
}

impl<'a> IRVisitor<()> for X86Builder<'a> {
    // TODO: sub rsp with aligned stack size
    fn visit_func(&mut self, f: &crate::ir::FuncInst) {
        if f.public {
            self.text.writeln(&format!(".globl {}", f.name));
        }

        self.text.writeln(&format!("{}:", f.name));
        self.push();

        self.text.writeln("push rbp");
        self.text.writeln("mov rbp, rsp");

        self.alloc.reset_params();
        for (i, ty) in f.params.iter().enumerate() {
            let dest = self.stack_alloc(ty.size());
            let reg = self.alloc.next_param_reg(ty);

            self.bind_param(i, &dest);
            self.mov_str(&dest, &reg, ty);
        }

        for ins in &f.body {
            ins.accept(self);
        }

        self.pop();
    }

    fn visit_ret(&mut self, ty: &crate::ir::Type, v: &crate::ir::Value) {
        // If not void
        if ty.size() != 0 {
            self.mov(&self.alloc.return_reg(ty), v, ty);
        }
        self.text.writeln("leave");
        self.text.writeln("ret\n");
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
                    self.mov(&dest, &arg.1, &arg.0);
                }
                self.text.writeln(&format!("call {}", name));
                self.bind(c.result, &self.alloc.return_reg(&c.ty));
            }
            _ => panic!("invalid call callee"),
        }
    }

    fn visit_static_string(&mut self, d: &crate::ir::StringDataIns) -> () {
        self.data
            .writeln(&format!(".{}: .asciz \"{}\"", d.name, d.value));
    }

    fn visit_extern(&mut self, f: &crate::ir::ExternFuncInst) -> () {
        self.head.writeln(&format!(".extern {}", f.name));
    }
}
