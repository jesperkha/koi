use std::mem::take;

use crate::{
    build::x86::{Asm, DataDecl, Dest, File, Immediate, Label, Reg, Size, Src, TextDecl},
    ir::{Data, Decl, ExternDecl, FuncDecl, IRTypeId, Ins, RValue, Unit},
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
}

impl<'a> FunctionAssembler<'a> {
    fn new(unit: &'a Unit, body: &'a [Ins]) -> Self {
        Self {
            unit,
            body,
            asm: Vec::new(),
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
            Ins::Store(store_ins) => todo!(),
            Ins::Assign(assign_ins) => todo!(),
            Ins::Call(call) => {}
            Ins::Intrinsic(intrinsic_ins) => todo!(),
            Ins::Return(ty, rvalue) => self.emit_return(ty, rvalue),
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
            RValue::Void => todo!(),
            RValue::Float(_) => todo!(),
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
