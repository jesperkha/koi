use std::{collections::HashMap, mem::take};

use crate::{
    build::x86::{
        Asm, DataDecl, Dest, File, Immediate, Label, Reg, Size, Src, StackOffset, TextDecl,
        UnsignedReg,
    },
    ir::{
        AssignIns, CallIns, ConstId, Data, Decl, ExternDecl, FuncDecl, IRType, IRTypeId, Ins,
        LValue, Primitive, RValue, StoreIns, Unit,
    },
};

static MIN_STACK_SIZE: usize = 4;

pub(crate) fn assemble(unit: Unit) -> File {
    let assembler = Assembler::new(unit);
    assembler.assemble()
}

pub(crate) struct Assembler {
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
        let fasm = FunctionAssembler::new(&self.unit, &decl);
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
    acc_offset: usize,
    params: Vec<Reg>,
    vars: HashMap<ConstId, Dest>,
}

impl<'a> FunctionAssembler<'a> {
    fn new(unit: &'a Unit, decl: &'a FuncDecl) -> Self {
        let mut regs = RegAllocator::new();
        let params = decl.params.iter().map(|ty| regs.next(unit, ty)).collect();
        Self {
            unit,
            body: &decl.body.ins,
            asm: Vec::new(),
            vars: HashMap::new(),
            acc_offset: 0,
            params,
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
        // Allocate more stack space
        self.acc_offset += self.unit.types.sizeof(store.ty).max(MIN_STACK_SIZE);

        // Map ConstId to stack offset
        let dest = Dest::StackOffset(StackOffset {
            offset: self.acc_offset,
            size: self.type_size(&store.ty),
        });
        self.vars.insert(store.const_id, dest.clone());

        // Move value into offset
        let src = self.rval_to_src(&store.rval);
        let src = self.src_to_movable(src, &store.ty);
        self.mov_or_lea(dest, src);
    }

    fn emit_assign(&mut self, assign: &AssignIns) {
        let src = self.rval_to_src(&assign.rval);
        let src = self.src_to_movable(src, &assign.ty);
        let dest = self.lval_to_dest(&assign.lval);
        self.mov_or_lea(dest, src);
    }

    fn emit_call(&mut self, call: &CallIns) {
        let mut regs = RegAllocator::new();

        // Move all arguments into call registers
        for (ty, rval) in &call.args {
            let src = self.rval_to_src(rval);
            let reg = regs.next(self.unit, ty);
            self.mov_or_lea(Dest::Reg(reg), src);
        }

        // call <name>
        match &call.callee {
            RValue::Function(name) => self.push(Asm::Call(name.clone())),
            RValue::Const(_) => todo!(),
            RValue::Param(_) => todo!(),
            _ => panic!("bad function callee kind"),
        }

        // If the return type is not void, assign it to a register
        if self.unit.types.get(call.ty).size() != 0 {
            match &call.result {
                LValue::Const(id) => {
                    self.vars.insert(*id, Dest::Reg(self.rax(&call.ty)));
                }
                // No need to allocate new register
                LValue::Param(_) => {}
            };
        }
    }

    fn emit_return(&mut self, ty: &IRTypeId, rval: &RValue) {
        // Move value into RAX if function returns a value
        if !matches!(rval, RValue::Void) {
            self.mov_or_lea(Dest::Reg(self.rax(ty)), self.rval_to_src(rval));
        }

        // leave
        // ret
        self.push(Asm::Leave);
        self.push(Asm::Ret);
    }

    /// Helper to automatically switch between mov and lea depending on value.
    fn mov_or_lea(&mut self, dest: Dest, src: Src) {
        self.push(if matches!(src, Src::Label(_)) {
            Asm::Lea(dest, src)
        } else {
            Asm::Mov(dest, src)
        });
    }

    /// Get parameter register
    fn param(&self, idx: usize) -> Reg {
        self.params[idx].clone()
    }

    /// Get variable dest (stack offset or register)
    fn var(&self, const_id: ConstId) -> &Dest {
        self.vars
            .get(&const_id)
            .expect(&format!("not stored: {const_id}"))
    }

    /// Convert IR RValue to a Src. May emit multiple steps to compute the final value.
    fn rval_to_src(&self, rval: &RValue) -> Src {
        match rval {
            RValue::Int(n) => Src::Immediate(Immediate::Int(*n)),
            RValue::Uint(n) => Src::Immediate(Immediate::Uint(*n)),
            RValue::Float(n) => Src::Immediate(Immediate::Float(*n)),
            RValue::Const(id) => self.var(*id).into(),
            RValue::Param(idx) => Src::Reg(self.param(*idx)),
            RValue::Data(idx) => Src::Label(Label {
                name: to_data_label(*idx),
            }),

            RValue::Void => todo!(),
            RValue::Function(_) => todo!(),
        }
    }

    /// Get the register or stack offset which the lvalue is located at.
    fn lval_to_dest(&mut self, lval: &LValue) -> Dest {
        match lval {
            LValue::Const(id) => self.var(*id).clone(),
            LValue::Param(idx) => Dest::Reg(self.param(*idx)),
        }
    }

    /// Convert Src to movable value (either immediate, register, or temp register RAX).
    fn src_to_movable(&mut self, src: Src, ty: &IRTypeId) -> Src {
        match src {
            Src::Reg(_) | Src::Immediate(_) => src,
            Src::StackOffset(_) | Src::Label(_) => {
                let rax = self.rax(ty);
                self.mov_or_lea(Dest::Reg(rax.clone()), src);
                Src::Reg(rax)
            }
        }
    }

    fn type_size(&self, ty: &IRTypeId) -> Size {
        type_size(self.unit, ty)
    }

    /// Shorthand for getting correctly sized RAX register
    fn rax(&self, ty: &IRTypeId) -> Reg {
        UnsignedReg::Rax.to_sized(self.type_size(ty))
    }
}

fn to_data_label(idx: usize) -> String {
    format!("D{}", idx)
}

struct RegAllocator {
    num_int: usize,
    num_float: usize,
}

static INT_REGS: [UnsignedReg; 6] = [
    UnsignedReg::Rdi,
    UnsignedReg::Rsi,
    UnsignedReg::Rdx,
    UnsignedReg::Rcx,
    UnsignedReg::R8,
    UnsignedReg::R9,
];
static FLOAT_REGS: [UnsignedReg; 8] = [
    UnsignedReg::Xmm0,
    UnsignedReg::Xmm1,
    UnsignedReg::Xmm2,
    UnsignedReg::Xmm3,
    UnsignedReg::Xmm4,
    UnsignedReg::Xmm5,
    UnsignedReg::Xmm6,
    UnsignedReg::Xmm7,
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
        .to_sized(type_size(unit, ty))
    }

    fn next_int(&mut self) -> UnsignedReg {
        if self.num_int >= INT_REGS.len() {
            panic!("exceeded maximum int registers");
        }
        let reg = INT_REGS[self.num_int].clone();
        self.num_int += 1;
        reg
    }

    fn next_float(&mut self) -> UnsignedReg {
        if self.num_float >= FLOAT_REGS.len() {
            panic!("exceeded maximum float registers");
        }
        let reg = FLOAT_REGS[self.num_float].clone();
        self.num_float += 1;
        reg
    }
}

fn type_size(unit: &Unit, ty: &IRTypeId) -> Size {
    match unit.types.sizeof(*ty) {
        1 => Size::Byte,
        2 => Size::Word,
        4 => Size::Dword,
        8 => Size::Qword,
        _ => panic!("no size"),
    }
}
