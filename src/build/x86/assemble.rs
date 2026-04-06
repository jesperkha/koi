use std::{collections::HashMap, mem::take};

use crate::{
    build::x86::{
        Asm, Condition, DataDecl, Dest, File, Immediate, Label, Reg, Size, Src, StackOffset,
        TextDecl, UnsignedReg,
    },
    config::Config,
    ir::{
        AssignIns, BinaryIns, Block, CallIns, ConstId, Data, Decl, ExternDecl, FuncDecl,
        IRBinaryOp, IRType, IRTypeId, IRUnaryOp, IfIns, Ins, LValue, Primitive, RValue, StoreIns,
        UnaryIns, Unit, WhileIns, ins_to_string_oneline,
    },
};

pub fn assemble(unit: Unit, config: &Config) -> File {
    let assembler = Assembler::new(unit, config);
    assembler.assemble()
}

pub struct Assembler<'a> {
    config: &'a Config,
    unit: Unit,
    data: Vec<DataDecl>,
    text: Vec<TextDecl>,
}

impl<'a> Assembler<'a> {
    pub fn new(unit: Unit, config: &'a Config) -> Self {
        Self {
            config,
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
        let fasm = FunctionAssembler::new(&self.unit, &decl, self.config);
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
    decl: &'a FuncDecl,
    config: &'a Config,

    asm: Vec<Asm>,
    acc_offset: usize,
    params: Vec<Dest>,
    vars: HashMap<ConstId, Dest>,

    /// Scoped pairs of label+end
    loop_labels: Vec<(String, String)>,

    // Number of conditional and end labels
    cond_count: usize,
    cond_end_count: usize,

    // Number of loop and end labels
    loop_end_count: usize,
    loop_count: usize,
}

impl<'a> FunctionAssembler<'a> {
    fn new(unit: &'a Unit, decl: &'a FuncDecl, config: &'a Config) -> Self {
        Self {
            unit,
            decl,
            config,
            asm: Vec::new(),
            vars: HashMap::new(),
            acc_offset: 0,
            params: Vec::new(),
            cond_count: 0,
            cond_end_count: 0,
            loop_count: 0,
            loop_end_count: 0,
            loop_labels: Vec::new(),
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

        let mut regs = RegAllocator::new();
        let params = self
            .decl
            .params
            .iter()
            .map(|ty| regs.next(self.unit, ty))
            .collect::<Vec<Reg>>();

        let param_stack_size = self
            .decl
            .params
            .iter()
            .fold(0_usize, |acc, ty| acc + self.sizeof(ty));

        // sub rsp, [x]
        let stacksize = round_to_16(self.decl.stacksize + param_stack_size);
        if stacksize != 0 {
            self.push(Asm::Sub(
                Dest::Reg(Reg::Rsp),
                Src::Immediate(Immediate::Uint(stacksize as u64)),
            ));
        }

        // Put parameters on stack
        for (i, ty) in self.decl.params.iter().enumerate() {
            let dest = self.new_stack_offset(ty);
            self.mov_or_lea(dest.clone(), Src::Reg(params[i].clone()));
            self.params.push(dest);
        }

        self.emit_block(&self.decl.body);
        self.asm
    }

    fn emit_block(&mut self, block: &Block) {
        for i in &block.ins {
            self.emit_ins(i);
        }
    }

    fn emit_ins(&mut self, ins: &Ins) {
        if self.config.comment_assembly {
            self.push(Asm::Comment(ins_to_string_oneline(self.unit, ins)));
        }

        match ins {
            Ins::Store(store) => self.emit_store(store),
            Ins::Assign(assign) => self.emit_assign(assign),
            Ins::Call(call) => self.emit_call(call),
            Ins::Return(ty, rvalue) => self.emit_return(ty, rvalue),
            Ins::Intrinsic(_) => todo!(),
            Ins::Binary(ins) => self.emit_binary(ins),
            Ins::Unary(ins) => self.emit_unary(ins),
            Ins::If(ins) => self.emit_if(ins),
            Ins::While(ins) => self.emit_while(ins),
            Ins::Break => self.emit_break(),
            Ins::Continue => self.emit_continue(),
        }
    }

    fn emit_break(&mut self) {
        let (_, end) = self.cur_loop_labels();
        self.push(Asm::Jmp(end.clone()));
    }

    fn emit_continue(&mut self) {
        let (start, _) = self.cur_loop_labels();
        self.push(Asm::Jmp(start.clone()));
    }

    fn evaluate_bool_and_cmp(&mut self, cond: &RValue) {
        let bool_src = self.rval_to_src(cond); // Evaluate condition
        if matches!(bool_src, Src::Immediate(_)) {
            // If bool_src is immediate it can only be either 1 or 0
            // In this case we just compare it with an empty AL to set the zero flag
            self.push(Asm::Xor(Dest::Reg(Reg::Al), Src::Reg(Reg::Al)));
            self.push(Asm::Cmp(Src::Reg(Reg::Al), bool_src));
        } else {
            self.push(Asm::Cmp(bool_src, Src::Immediate(Immediate::Uint(0))));
        }
    }

    fn emit_while(&mut self, ins: &WhileIns) {
        let label = self.next_loop_label();
        let end = self.next_loop_end_label();

        self.push(Asm::Label(label.clone()));

        // Eval and jump if false
        for i in &ins.cond_ins {
            self.emit_ins(i);
        }
        self.evaluate_bool_and_cmp(&ins.cond);
        self.push(Asm::Jz(end.clone()));

        // Run ins and jump back
        self.push_loop_labels(label.clone(), end.clone());
        self.emit_block(&ins.block);
        self.push(Asm::Jmp(label));
        self.pop_loop_labels();

        // Finish
        self.push(Asm::Label(end));
    }

    fn emit_if(&mut self, ifins: &IfIns) {
        let end = self.next_cond_end_label();
        let has_branches = !ifins.elseif.is_empty() || ifins.elseblock.is_some();

        // Allocate the label for the first else-if or else branch
        let mut next_label = if has_branches {
            Some(self.next_cond_label())
        } else {
            None
        };

        // Emit if block; jump to next branch on false, or end if no branches
        self.evaluate_bool_and_cmp(&ifins.cond);
        self.push(Asm::Jz(next_label.clone().unwrap_or(end.clone())));
        self.emit_block(&ifins.block);
        self.push(Asm::Jmp(end.clone()));

        // Emit else-if branches; each one consumes next_label and allocates the
        // label for the branch that follows it
        for (idx, elseif) in ifins.elseif.iter().enumerate() {
            self.push(Asm::Label(next_label.take().unwrap()));

            let is_last = idx == ifins.elseif.len() - 1;
            next_label = if is_last && ifins.elseblock.is_none() {
                None // false falls through to end
            } else {
                Some(self.next_cond_label()) // label for next else-if or else
            };

            for i in &elseif.cond_ins {
                self.emit_ins(i);
            }
            self.evaluate_bool_and_cmp(&elseif.cond);
            self.push(Asm::Jz(next_label.clone().unwrap_or(end.clone())));
            self.emit_block(&elseif.block);
            self.push(Asm::Jmp(end.clone()));
        }

        if let Some(elseblock) = &ifins.elseblock {
            self.push(Asm::Label(next_label.unwrap()));
            self.emit_block(elseblock);
        }

        self.push(Asm::Label(end));
    }

    fn emit_store(&mut self, store: &StoreIns) {
        // Map ConstId to stack offset
        let dest = self.new_stack_offset(&store.ty);
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

    fn emit_binary(&mut self, ins: &BinaryIns) {
        let lhs = self.rval_to_src(&ins.lhs);
        let rhs = self.rval_to_src(&ins.rhs);
        let result_size = self.type_size(&ins.ty);
        let rax = UnsignedReg::Rax.to_sized(result_size.clone());

        match &ins.op {
            IRBinaryOp::Add | IRBinaryOp::Sub | IRBinaryOp::Mul => {
                let r10 = UnsignedReg::R10.to_sized(result_size);
                self.push(Asm::Mov(Dest::Reg(r10.clone()), rhs));
                self.push(Asm::Mov(Dest::Reg(rax.clone()), lhs));
                let op = match &ins.op {
                    IRBinaryOp::Add => Asm::Add(Dest::Reg(rax.clone()), Src::Reg(r10)),
                    IRBinaryOp::Sub => Asm::Sub(Dest::Reg(rax.clone()), Src::Reg(r10)),
                    IRBinaryOp::Mul => Asm::IMul(Dest::Reg(rax.clone()), Src::Reg(r10)),
                    _ => unreachable!(),
                };
                self.push(op);
                self.vars.insert(ins.result, Dest::Reg(rax));
            }
            IRBinaryOp::Div => {
                let r10 = UnsignedReg::R10.to_sized(result_size.clone());
                self.push(Asm::Mov(Dest::Reg(r10.clone()), rhs));
                self.push(Asm::Mov(Dest::Reg(rax.clone()), lhs));
                self.push(sign_extend_ax(&result_size));
                self.push(Asm::IDiv(Src::Reg(r10)));
                self.vars.insert(ins.result, Dest::Reg(rax));
            }
            IRBinaryOp::Mod => {
                // Result is u32; operand size is derived from lhs.
                let op_size = src_size(&lhs);
                let rax_op = UnsignedReg::Rax.to_sized(op_size.clone());
                let r10_op = UnsignedReg::R10.to_sized(op_size.clone());
                self.push(Asm::Mov(Dest::Reg(r10_op.clone()), rhs));
                self.push(Asm::Mov(Dest::Reg(rax_op.clone()), lhs));
                self.push(sign_extend_ax(&op_size));
                self.push(Asm::IDiv(Src::Reg(r10_op)));
                self.vars.insert(ins.result, Dest::Reg(Reg::Edx));
            }
            IRBinaryOp::Eq
            | IRBinaryOp::Ne
            | IRBinaryOp::Gt
            | IRBinaryOp::Ge
            | IRBinaryOp::Lt
            | IRBinaryOp::Le => {
                // Determine operand size from the non-immediate src; both operands have the same type.
                let op_size = match (&lhs, &rhs) {
                    (Src::Immediate(_), Src::Immediate(_)) => Size::Qword,
                    (Src::Immediate(_), _) => src_size(&rhs),
                    _ => src_size(&lhs),
                };
                let rax_op = UnsignedReg::Rax.to_sized(op_size.clone());
                let r10_op = UnsignedReg::R10.to_sized(op_size);
                self.push(Asm::Mov(Dest::Reg(r10_op.clone()), rhs));
                self.push(Asm::Mov(Dest::Reg(rax_op.clone()), lhs));
                self.push(Asm::Cmp(Src::Reg(rax_op), Src::Reg(r10_op)));
                let cond = match &ins.op {
                    IRBinaryOp::Eq => Condition::E,
                    IRBinaryOp::Ne => Condition::Ne,
                    IRBinaryOp::Gt => Condition::G,
                    IRBinaryOp::Ge => Condition::Ge,
                    IRBinaryOp::Lt => Condition::L,
                    IRBinaryOp::Le => Condition::Le,
                    _ => unreachable!(),
                };
                self.push(Asm::Set(cond, Dest::Reg(Reg::Al)));
                self.vars.insert(ins.result, Dest::Reg(Reg::Al));
            }
            IRBinaryOp::And | IRBinaryOp::Or => {
                let r10 = UnsignedReg::R10.to_sized(result_size);
                self.push(Asm::Mov(Dest::Reg(r10.clone()), rhs));
                self.push(Asm::Mov(Dest::Reg(rax.clone()), lhs));
                let op = match &ins.op {
                    IRBinaryOp::And => Asm::And(Dest::Reg(rax.clone()), Src::Reg(r10)),
                    IRBinaryOp::Or => Asm::Or(Dest::Reg(rax.clone()), Src::Reg(r10)),
                    _ => unreachable!(),
                };
                self.push(op);
                self.vars.insert(ins.result, Dest::Reg(rax));
            }
        }
    }

    fn emit_unary(&mut self, ins: &UnaryIns) {
        let rhs = self.rval_to_src(&ins.rhs);
        let result_size = self.type_size(&ins.ty);
        let rax = UnsignedReg::Rax.to_sized(result_size);

        match &ins.op {
            IRUnaryOp::Neg => {
                self.push(Asm::Mov(Dest::Reg(rax.clone()), rhs));
                self.push(Asm::Neg(Dest::Reg(rax.clone())));
                self.vars.insert(ins.result, Dest::Reg(rax));
            }
            IRUnaryOp::Not => {
                // Boolean not: flip the low bit (xor with 1)
                self.push(Asm::Mov(Dest::Reg(rax.clone()), rhs));
                self.push(Asm::Xor(
                    Dest::Reg(rax.clone()),
                    Src::Immediate(Immediate::Uint(1)),
                ));
                self.vars.insert(ins.result, Dest::Reg(rax));
            }
        }
    }

    //      HELPER METHODS
    // ----------------------------

    fn next_cond_end_label(&mut self) -> String {
        let l = format!(".L{}_cond_end_{}", self.decl.name, self.cond_end_count);
        self.cond_end_count += 1;
        l
    }

    fn next_cond_label(&mut self) -> String {
        let l = format!(".L{}_cond_{}", self.decl.name, self.cond_count);
        self.cond_count += 1;
        l
    }

    fn next_loop_end_label(&mut self) -> String {
        let l = format!(".L{}_loop_end_{}", self.decl.name, self.loop_end_count);
        self.loop_end_count += 1;
        l
    }

    fn next_loop_label(&mut self) -> String {
        let l = format!(".L{}_loop_{}", self.decl.name, self.loop_count);
        self.loop_count += 1;
        l
    }

    fn push_loop_labels(&mut self, label: String, end: String) {
        self.loop_labels.push((label, end));
    }

    fn pop_loop_labels(&mut self) -> (String, String) {
        self.loop_labels.pop().unwrap()
    }

    fn cur_loop_labels(&self) -> &(String, String) {
        self.loop_labels.last().unwrap()
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
    fn param(&self, idx: usize) -> Dest {
        self.params[idx].clone()
    }

    /// Get variable dest (stack offset or register)
    fn var(&self, const_id: ConstId) -> &Dest {
        self.vars
            .get(&const_id)
            .unwrap_or_else(|| panic!("not stored: {const_id}"))
    }

    /// Convert IR RValue to a Src. May emit multiple steps to compute the final value.
    fn rval_to_src(&self, rval: &RValue) -> Src {
        match rval {
            RValue::Int(n) => Src::Immediate(Immediate::Int(*n)),
            RValue::Uint(n) => Src::Immediate(Immediate::Uint(*n)),
            RValue::Float(n) => Src::Immediate(Immediate::Float(*n)),
            RValue::Const(id) => self.var(*id).into(),
            RValue::Param(idx) => (&self.param(*idx)).into(),
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
            LValue::Param(idx) => self.param(*idx),
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

    /// Allocate new stack slot for variable
    fn new_stack_offset(&mut self, ty: &IRTypeId) -> Dest {
        self.acc_offset += self.sizeof(ty);

        Dest::StackOffset(StackOffset {
            offset: self.acc_offset,
            size: self.type_size(ty),
        })
    }

    fn type_size(&self, ty: &IRTypeId) -> Size {
        type_size(self.unit, ty)
    }

    fn sizeof(&self, ty: &IRTypeId) -> usize {
        self.unit.types.sizeof(*ty)
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

/// Emit the correct sign-extend-into-rdx instruction for idiv based on operand size.
fn sign_extend_ax(size: &Size) -> Asm {
    match size {
        Size::Qword => Asm::Cqo,
        _ => Asm::Cdq,
    }
}

fn round_to_16(n: usize) -> usize {
    (n + 15) & !15
}

/// Get the operand size of a Src value.
fn src_size(src: &Src) -> Size {
    match src {
        Src::Reg(reg) => reg_to_size(reg),
        Src::StackOffset(off) => off.size.clone(),
        Src::Immediate(_) | Src::Label(_) => Size::Qword,
    }
}

fn reg_to_size(reg: &Reg) -> Size {
    match reg {
        Reg::Rax
        | Reg::Rbx
        | Reg::Rcx
        | Reg::Rdx
        | Reg::Rbp
        | Reg::Rsp
        | Reg::Rsi
        | Reg::Rdi
        | Reg::R8
        | Reg::R9
        | Reg::R10
        | Reg::R11
        | Reg::R12
        | Reg::R13
        | Reg::R14
        | Reg::R15 => Size::Qword,
        Reg::Eax
        | Reg::Ebx
        | Reg::Ecx
        | Reg::Edx
        | Reg::Esi
        | Reg::Edi
        | Reg::R8d
        | Reg::R9d
        | Reg::R10d
        | Reg::R11d
        | Reg::R12d
        | Reg::R13d
        | Reg::R14d
        | Reg::R15d => Size::Dword,
        Reg::Ax
        | Reg::Bx
        | Reg::Cx
        | Reg::Ex
        | Reg::Si
        | Reg::Di
        | Reg::R8w
        | Reg::R9w
        | Reg::R10w
        | Reg::R11w
        | Reg::R12w
        | Reg::R13w
        | Reg::R14w
        | Reg::R15w => Size::Word,
        Reg::Al
        | Reg::Bl
        | Reg::Cl
        | Reg::El
        | Reg::Sil
        | Reg::Dil
        | Reg::R8b
        | Reg::R9b
        | Reg::R10b
        | Reg::R11b
        | Reg::R12b
        | Reg::R13b
        | Reg::R14b
        | Reg::R15b => Size::Byte,
        Reg::Xmm0
        | Reg::Xmm1
        | Reg::Xmm2
        | Reg::Xmm3
        | Reg::Xmm4
        | Reg::Xmm5
        | Reg::Xmm6
        | Reg::Xmm7 => Size::Qword,
    }
}
