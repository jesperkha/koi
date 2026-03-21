use std::{collections::HashMap, mem::take};

use crate::{
    build::x86::{Asm, DataDecl, Dest, File, Immediate, Label, Reg, Size, Src, TextDecl},
    ir::{
        AssignIns, CallIns, ConstId, Data, Decl, ExternDecl, FuncDecl, IRType, IRTypeId, Ins,
        Primitive, RValue, StoreIns, Unit,
    },
};

pub fn assemble(unit: Unit) -> File {
    let assembler = Assembler::new(unit);
    assembler.assemble()
}

pub struct Assembler {
    unit: Unit,
    data: Vec<DataDecl>,
    text: Vec<TextDecl>,
}

impl Assembler {
    pub fn new(unit: Unit) -> Self {
        Self {
            unit,
            data: Vec::new(),
            text: Vec::new(),
        }
    }

    pub fn assemble(mut self) -> File {
        let decls = take(&mut self.unit.decls);
        let data = take(&mut self.unit.data);

        // Assemble all declarations
        for decl in decls {
            let text_decl = match decl {
                Decl::Extern(decl) => self.emit_extern(decl),
                Decl::Func(decl) => self.emit_func(decl),
            };

            self.text.push(text_decl);
        }

        // Declare all data segments
        for (i, data) in data.into_iter().enumerate() {
            let data_decl = match data {
                Data::String(s) => DataDecl::String {
                    label: to_data_label(i),
                    content: s.clone(),
                },
            };

            self.data.push(data_decl);
        }

        File {
            data_section: self.data,
            text_section: self.text,
        }
    }

    fn emit_extern(&mut self, decl: ExternDecl) -> TextDecl {
        TextDecl::Extern(decl.name.clone())
    }

    fn emit_func(&mut self, decl: FuncDecl) -> TextDecl {
        let fasm = FunctionAssembler::new(&self.unit, &decl.body.ins);
        let ins = fasm.assemble();

        TextDecl::Function {
            global: decl.public,
            name: decl.name.clone(),
            ins,
        }
    }
}

struct FunctionAssembler<'a> {
    unit: &'a Unit,
    body: &'a [Ins],
    asm: Vec<Asm>,
    vars: HashMap<ConstId, Dest>,
}

impl<'a> FunctionAssembler<'a> {
    fn new(unit: &'a Unit, body: &'a [Ins]) -> Self {
        Self {
            unit,
            body,
            asm: Vec::new(),
            vars: HashMap::new(),
        }
    }

    fn push(&mut self, asm: Asm) {
        self.asm.push(asm);
    }

    fn assemble(mut self) -> Vec<Asm> {
        // push rbp
        // mov rbp, rsp
        self.push(Asm::Push(Src::Reg(Reg::Rbp)));
        self.push(Asm::Mov(Dest::Reg(Reg::Rbp), Src::Reg(Reg::Rsp)));

        for i in self.body {
            self.emit_ins(i);
        }

        self.asm
    }

    fn emit_ins(&mut self, ins: &Ins) {
        match ins {
            Ins::Store(store) => self.emit_store(store),
            Ins::Assign(assign) => self.emit_assign(assign),
            Ins::Call(call) => self.emit_call(call),
            Ins::Return(ty, rvalue) => self.emit_return(ty, rvalue),
            Ins::Intrinsic(_) => todo!(),
        }
    }

    fn emit_store(&mut self, store: &StoreIns) {
        todo!()
    }

    fn emit_assign(&mut self, assign: &AssignIns) {
        todo!()
    }

    fn emit_call(&mut self, call: &CallIns) {
        let mut regs = RegAllocator::new();

        for (ty, rval) in &call.args {
            let src = self.rval_to_src(rval);
            let reg = regs.next(self.unit, ty);
            self.mov_or_lea(Dest::Reg(reg), src);
        }

        match &call.callee {
            RValue::Function(name) => self.push(Asm::Call(name.clone())),
            RValue::Const(_) => todo!(),
            RValue::Param(_) => todo!(),
            _ => panic!("bad function callee kind"),
        }
    }

    fn emit_return(&mut self, ty: &IRTypeId, rval: &RValue) {
        if !matches!(rval, RValue::Void) {
            // mov/lea rax, [...]
            self.mov_or_lea(Dest::Reg(Reg::Rax), self.rval_to_src(rval));
        }

        // leave
        // ret
        self.push(Asm::Leave);
        self.push(Asm::Ret);
    }

    fn mov_or_lea(&mut self, dest: Dest, src: Src) {
        self.push(if matches!(src, Src::Label(_)) {
            Asm::Lea(dest, src)
        } else {
            Asm::Mov(dest, src)
        });
    }

    fn rval_to_src(&self, rval: &RValue) -> Src {
        match rval {
            RValue::Int(n) => Src::Immediate(Immediate::Int(*n)),
            RValue::Uint(n) => Src::Immediate(Immediate::Uint(*n)),
            RValue::Float(n) => Src::Immediate(Immediate::Float(*n)),
            RValue::Void => todo!(),
            RValue::Const(_) => todo!(),
            RValue::Param(_) => todo!(),
            RValue::Function(_) => todo!(),
            RValue::Data(idx) => Src::Label(Label {
                name: to_data_label(*idx),
            }),
        }
    }

    fn type_size(&self, ty: &IRTypeId) -> Size {
        match self.unit.types.sizeof(*ty) {
            1 => Size::Byte,
            2 => Size::Word,
            4 => Size::Dword,
            8 => Size::Qword,
            _ => panic!("no size"),
        }
    }
}

fn to_data_label(idx: usize) -> String {
    format!("D{}", idx)
}

struct RegAllocator {
    num_int: usize,
    num_float: usize,
}

static INT_REGS: [Reg; 6] = [Reg::Rdi, Reg::Rsi, Reg::Rdx, Reg::Rcx, Reg::R8, Reg::R9];
static FLOAT_REGS: [Reg; 8] = [
    Reg::Xmm0,
    Reg::Xmm1,
    Reg::Xmm2,
    Reg::Xmm3,
    Reg::Xmm4,
    Reg::Xmm5,
    Reg::Xmm6,
    Reg::Xmm7,
];

impl RegAllocator {
    fn new() -> Self {
        Self {
            num_int: 0,
            num_float: 0,
        }
    }

    fn next(&mut self, unit: &Unit, ty: &IRTypeId) -> Reg {
        match unit.types.get(*ty) {
            IRType::Primitive(primitive) => match primitive {
                Primitive::F32 | Primitive::F64 => self.next_float(),
                Primitive::U8
                | Primitive::U16
                | Primitive::U32
                | Primitive::U64
                | Primitive::I8
                | Primitive::I16
                | Primitive::I32
                | Primitive::I64
                | Primitive::String => self.next_int(),
                Primitive::Void => panic!("void type not allowed"),
            },
            IRType::Function(..) => todo!(),
        }
    }

    fn next_int(&mut self) -> Reg {
        if self.num_int >= INT_REGS.len() {
            panic!("exceeded maximum int registers");
        }
        let reg = INT_REGS[self.num_int].clone();
        self.num_int += 1;
        reg
    }

    fn next_float(&mut self) -> Reg {
        if self.num_float >= FLOAT_REGS.len() {
            panic!("exceeded maximum float registers");
        }
        let reg = FLOAT_REGS[self.num_float].clone();
        self.num_float += 1;
        reg
    }
}
