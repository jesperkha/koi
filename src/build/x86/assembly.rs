use std::fmt::Display;

pub struct File {
    pub data_section: Vec<DataDecl>,
    pub rodata_section: Vec<RodataDecl>,
    pub text_section: Vec<TextDecl>,
}

pub enum DataDecl {
    /// An auto-labeled string (e.g. `.D0: .asciz "hello"`) used for anonymous string literals.
    String { label: String, content: String },
    /// A named, optionally-global string (e.g. `.globl sym\nsym: .asciz "hello"`).
    NamedString { global: bool, name: String, content: String },
}

/// A read-only data declaration emitted into `.rodata`.
pub enum RodataDecl {
    /// An integer, bool, or char constant.
    Integer {
        global: bool,
        name: String,
        /// Size in bytes: 1, 2, 4, or 8.
        bytes: usize,
        value: i64,
    },
    /// A 64-bit float constant.
    Float64 {
        global: bool,
        name: String,
        value: f64,
    },
    /// A 32-bit float constant.
    Float32 {
        global: bool,
        name: String,
        value: f32,
    },
}

pub enum TextDecl {
    Extern(String),
    Function {
        global: bool,
        name: String,
        ins: Vec<Asm>,
    },
}

pub enum Asm {
    Comment(String),
    Push(Src),
    Mov(Dest, Src),
    Lea(Dest, Src),
    Add(Dest, Src),
    Sub(Dest, Src),
    IMul(Dest, Src),
    IDiv(Src),
    Cqo,
    Cdq,
    Neg(Dest),
    Xor(Dest, Src),
    Cmp(Src, Src),
    Set(Condition, Dest),
    Call(String),
    Leave,
    Ret,
    Jmp(String),
    Jz(String),
    Jnz(String),
    Label(String),
}

pub enum Condition {
    E,
    Ne,
    G,
    Ge,
    L,
    Le,
}

#[derive(Clone, Debug)]
pub enum Size {
    Byte,
    Word,
    Dword,
    Qword,
}

pub enum Src {
    Immediate(Immediate),
    Reg(Reg),
    StackOffset(StackOffset),
    Label(Label),
}

impl From<&Dest> for Src {
    fn from(dest: &Dest) -> Self {
        match dest {
            Dest::Reg(reg) => Src::Reg(reg.clone()),
            Dest::StackOffset(stack) => Src::StackOffset(stack.clone()),
        }
    }
}

pub enum Immediate {
    Int(i64),
    Uint(u64),
    Float(f64),
}

#[derive(Clone)]
pub enum Dest {
    Reg(Reg),
    StackOffset(StackOffset),
}

#[derive(Debug, Clone)]
pub enum Reg {
    Rax,
    Eax,
    Ax,
    Al,

    Rcx,
    Ecx,
    Cx,
    Cl,

    Rdx,
    Edx,
    Ex,
    El,

    Rbx,
    Ebx,
    Bx,
    Bl,

    Rsi,
    Esi,
    Si,
    Sil,

    Rdi,
    Edi,
    Di,
    Dil,

    R8,
    R8d,
    R8w,
    R8b,

    R9,
    R9d,
    R9w,
    R9b,

    R10,
    R10d,
    R10w,
    R10b,

    R11,
    R11d,
    R11w,
    R11b,

    R12,
    R12d,
    R12w,
    R12b,

    R13,
    R13d,
    R13w,
    R13b,

    R14,
    R14d,
    R14w,
    R14b,

    R15,
    R15d,
    R15w,
    R15b,

    Rbp,
    Rsp,

    Xmm0,
    Xmm1,
    Xmm2,
    Xmm3,
    Xmm4,
    Xmm5,
    Xmm6,
    Xmm7,
}

#[derive(Debug, Clone)]
pub enum UnsignedReg {
    Rax,

    Rbp,
    Rsp,
    Rbx,

    // Integer parameters
    Rdi,
    Rsi,
    Rdx,
    Rcx,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,

    // Floating point parameters
    Xmm0,
    Xmm1,
    Xmm2,
    Xmm3,
    Xmm4,
    Xmm5,
    Xmm6,
    Xmm7,
}

#[derive(Clone)]
pub struct StackOffset {
    pub offset: usize,
    pub size: Size,
}

pub struct Label {
    pub name: String,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, ".intel_syntax noprefix")?;

        write!(f, ".section .data\n\n")?;
        for decl in &self.data_section {
            writeln!(f, "{}", decl)?;
        }

        if !self.rodata_section.is_empty() {
            write!(f, ".section .rodata\n\n")?;
            for decl in &self.rodata_section {
                writeln!(f, "{}", decl)?;
            }
        }

        write!(f, ".section .text\n\n")?;
        for decl in &self.text_section {
            writeln!(f, "{}", decl)?;
        }

        writeln!(f, ".section .note.GNU-stack,\"\",@progbits")?;
        Ok(())
    }
}

impl Display for DataDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataDecl::String { label, content } => {
                write!(f, ".{}: .asciz \"{}\"", label, content)
            }
            DataDecl::NamedString { global, name, content } => {
                if *global {
                    writeln!(f, ".globl {}", name)?;
                }
                write!(f, "{}: .asciz \"{}\"", name, content)
            }
        }
    }
}

impl Display for RodataDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RodataDecl::Integer { global, name, bytes, value } => {
                if *global {
                    writeln!(f, ".globl {}", name)?;
                }
                let directive = match bytes {
                    1 => ".byte",
                    2 => ".short",
                    4 => ".long",
                    8 => ".quad",
                    _ => panic!("unsupported const size: {}", bytes),
                };
                write!(f, "{}:\n    {} {}", name, directive, value)
            }
            RodataDecl::Float64 { global, name, value } => {
                if *global {
                    writeln!(f, ".globl {}", name)?;
                }
                write!(f, "{}:\n    .double {}", name, value)
            }
            RodataDecl::Float32 { global, name, value } => {
                if *global {
                    writeln!(f, ".globl {}", name)?;
                }
                write!(f, "{}:\n    .float {}", name, value)
            }
        }
    }
}

impl Display for TextDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextDecl::Function { global, name, ins } => {
                if *global {
                    writeln!(f, ".globl {}", name)?;
                }
                writeln!(f, "{}:", name)?;
                for i in ins {
                    writeln!(f, "    {}", i)?;
                }
                Ok(())
            }
            TextDecl::Extern(name) => write!(f, ".extern {}", name),
        }
    }
}

impl Display for Asm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Asm::Comment(comment) => write!(f, "/* {} */", comment),
            Asm::Mov(dst, src) => write!(f, "mov {}, {}", dst, src),
            Asm::Lea(dst, src) => write!(f, "lea {}, {}", dst, src),
            Asm::Add(dst, src) => write!(f, "add {}, {}", dst, src),
            Asm::Sub(dst, src) => write!(f, "sub {}, {}", dst, src),
            Asm::IMul(dst, src) => write!(f, "imul {}, {}", dst, src),
            Asm::IDiv(src) => write!(f, "idiv {}", src),
            Asm::Cqo => write!(f, "cqo"),
            Asm::Cdq => write!(f, "cdq"),
            Asm::Neg(dst) => write!(f, "neg {}", dst),
            Asm::Xor(dst, src) => write!(f, "xor {}, {}", dst, src),
            Asm::Cmp(src1, src2) => write!(f, "cmp {}, {}", src1, src2),
            Asm::Set(cond, dst) => write!(f, "set{} {}", cond, dst),
            Asm::Push(source) => write!(f, "push {}", source),
            Asm::Leave => write!(f, "leave"),
            Asm::Ret => write!(f, "ret"),
            Asm::Call(label) => write!(f, "call {}", label),
            Asm::Jmp(label) => write!(f, "jmp {}", label),
            Asm::Jz(label) => write!(f, "jz {}", label),
            Asm::Jnz(label) => write!(f, "jnz {}", label),
            Asm::Label(label) => write!(f, "{}:", label),
        }
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Condition::E => "e",
            Condition::Ne => "ne",
            Condition::G => "g",
            Condition::Ge => "ge",
            Condition::L => "l",
            Condition::Le => "le",
        };
        write!(f, "{}", s)
    }
}

impl Display for StackOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} PTR [rbp-{}]", self.size, self.offset)
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use Debug to get the variant name, then uppercase it for assembly style
        let name = format!("{:?}", self).to_uppercase();
        write!(f, "{}", name)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ".{}", self.name)
    }
}

impl Display for Src {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Src::Reg(reg) => write!(f, "{}", reg),
            Src::StackOffset(stack) => write!(f, "{}", stack),
            Src::Label(label) => write!(f, "[rip + {}]", label),
            Src::Immediate(imm) => write!(f, "{}", imm),
        }
    }
}

impl Display for Immediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Immediate::Int(n) => n.to_string(),
                Immediate::Uint(n) => n.to_string(),
                Immediate::Float(n) => n.to_string(),
            }
        )
    }
}

impl Display for Dest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dest::Reg(reg) => write!(f, "{}", reg),
            Dest::StackOffset(stack) => write!(f, "{}", stack),
        }
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use Debug to get the variant name, then lowercase it for assembly style
        let name = format!("{:?}", self).to_lowercase();
        write!(f, "{}", name)
    }
}

impl From<Reg> for UnsignedReg {
    fn from(value: Reg) -> Self {
        match value {
            Reg::Rbx | Reg::Ebx | Reg::Bx | Reg::Bl => UnsignedReg::Rbx,
            Reg::Rax | Reg::Eax | Reg::Ax | Reg::Al => UnsignedReg::Rax,
            Reg::Rdi | Reg::Edi | Reg::Di | Reg::Dil => UnsignedReg::Rdi,
            Reg::Rsi | Reg::Esi | Reg::Si | Reg::Sil => UnsignedReg::Rsi,
            Reg::Rdx | Reg::Edx | Reg::Ex | Reg::El => UnsignedReg::Rdx,
            Reg::Rcx | Reg::Ecx | Reg::Cx | Reg::Cl => UnsignedReg::Rcx,
            Reg::R8 | Reg::R8d | Reg::R8w | Reg::R8b => UnsignedReg::R8,
            Reg::R9 | Reg::R9d | Reg::R9w | Reg::R9b => UnsignedReg::R9,
            Reg::R10 | Reg::R10d | Reg::R10w | Reg::R10b => UnsignedReg::R10,
            Reg::R11 | Reg::R11d | Reg::R11w | Reg::R11b => UnsignedReg::R11,
            Reg::R12 | Reg::R12d | Reg::R12w | Reg::R12b => UnsignedReg::R12,
            Reg::R13 | Reg::R13d | Reg::R13w | Reg::R13b => UnsignedReg::R13,
            Reg::R14 | Reg::R14d | Reg::R14w | Reg::R14b => UnsignedReg::R14,
            Reg::R15 | Reg::R15d | Reg::R15w | Reg::R15b => UnsignedReg::R15,
            Reg::Rbp => UnsignedReg::Rbp,
            Reg::Rsp => UnsignedReg::Rsp,
            Reg::Xmm0 => UnsignedReg::Xmm0,
            Reg::Xmm1 => UnsignedReg::Xmm1,
            Reg::Xmm2 => UnsignedReg::Xmm2,
            Reg::Xmm3 => UnsignedReg::Xmm3,
            Reg::Xmm4 => UnsignedReg::Xmm4,
            Reg::Xmm5 => UnsignedReg::Xmm5,
            Reg::Xmm6 => UnsignedReg::Xmm6,
            Reg::Xmm7 => UnsignedReg::Xmm7,
        }
    }
}

impl UnsignedReg {
    pub fn to_sized(&self, size: Size) -> Reg {
        match size {
            Size::Byte => match self {
                UnsignedReg::Rax => Reg::Al,
                UnsignedReg::Rbp => Reg::Bl, // No direct 8-bit RBP, use BL as placeholder or error
                UnsignedReg::Rsp => Reg::Bl, // No direct 8-bit RSP, use BL as placeholder or error
                UnsignedReg::Rbx => Reg::Bl,
                UnsignedReg::Rdi => Reg::Dil,
                UnsignedReg::Rsi => Reg::Sil,
                UnsignedReg::Rdx => Reg::El,
                UnsignedReg::Rcx => Reg::Cl,
                UnsignedReg::R8 => Reg::R8b,
                UnsignedReg::R9 => Reg::R9b,
                UnsignedReg::R10 => Reg::R10b,
                UnsignedReg::R11 => Reg::R11b,
                UnsignedReg::R12 => Reg::R12b,
                UnsignedReg::R13 => Reg::R13b,
                UnsignedReg::R14 => Reg::R14b,
                UnsignedReg::R15 => Reg::R15b,
                UnsignedReg::Xmm0 => panic!("No 8-bit XMM register"),
                UnsignedReg::Xmm1 => panic!("No 8-bit XMM register"),
                UnsignedReg::Xmm2 => panic!("No 8-bit XMM register"),
                UnsignedReg::Xmm3 => panic!("No 8-bit XMM register"),
                UnsignedReg::Xmm4 => panic!("No 8-bit XMM register"),
                UnsignedReg::Xmm5 => panic!("No 8-bit XMM register"),
                UnsignedReg::Xmm6 => panic!("No 8-bit XMM register"),
                UnsignedReg::Xmm7 => panic!("No 8-bit XMM register"),
            },
            Size::Word => match self {
                UnsignedReg::Rax => Reg::Ax,
                UnsignedReg::Rbp => Reg::Bx, // No direct 16-bit RBP, use BX as placeholder or error
                UnsignedReg::Rsp => Reg::Bx, // No direct 16-bit RSP, use BX as placeholder or error
                UnsignedReg::Rbx => Reg::Bx,
                UnsignedReg::Rdi => Reg::Di,
                UnsignedReg::Rsi => Reg::Si,
                UnsignedReg::Rdx => Reg::Ex, // No direct 16-bit RDX, use EX as placeholder or error
                UnsignedReg::Rcx => Reg::Cx,
                UnsignedReg::R8 => Reg::R8w,
                UnsignedReg::R9 => Reg::R9w,
                UnsignedReg::R10 => Reg::R10w,
                UnsignedReg::R11 => Reg::R11w,
                UnsignedReg::R12 => Reg::R12w,
                UnsignedReg::R13 => Reg::R13w,
                UnsignedReg::R14 => Reg::R14w,
                UnsignedReg::R15 => Reg::R15w,
                UnsignedReg::Xmm0 => panic!("No 16-bit XMM register"),
                UnsignedReg::Xmm1 => panic!("No 16-bit XMM register"),
                UnsignedReg::Xmm2 => panic!("No 16-bit XMM register"),
                UnsignedReg::Xmm3 => panic!("No 16-bit XMM register"),
                UnsignedReg::Xmm4 => panic!("No 16-bit XMM register"),
                UnsignedReg::Xmm5 => panic!("No 16-bit XMM register"),
                UnsignedReg::Xmm6 => panic!("No 16-bit XMM register"),
                UnsignedReg::Xmm7 => panic!("No 16-bit XMM register"),
            },
            Size::Dword => match self {
                UnsignedReg::Rax => Reg::Eax,
                UnsignedReg::Rbp => Reg::Ebx, // No direct 32-bit RBP, use EBX as placeholder or error
                UnsignedReg::Rsp => Reg::Ebx, // No direct 32-bit RSP, use EBX as placeholder or error
                UnsignedReg::Rbx => Reg::Ebx,
                UnsignedReg::Rdi => Reg::Edi,
                UnsignedReg::Rsi => Reg::Esi,
                UnsignedReg::Rdx => Reg::Edx,
                UnsignedReg::Rcx => Reg::Ecx,
                UnsignedReg::R8 => Reg::R8d,
                UnsignedReg::R9 => Reg::R9d,
                UnsignedReg::R10 => Reg::R10d,
                UnsignedReg::R11 => Reg::R11d,
                UnsignedReg::R12 => Reg::R12d,
                UnsignedReg::R13 => Reg::R13d,
                UnsignedReg::R14 => Reg::R14d,
                UnsignedReg::R15 => Reg::R15d,
                UnsignedReg::Xmm0 => panic!("No 32-bit XMM register"),
                UnsignedReg::Xmm1 => panic!("No 32-bit XMM register"),
                UnsignedReg::Xmm2 => panic!("No 32-bit XMM register"),
                UnsignedReg::Xmm3 => panic!("No 32-bit XMM register"),
                UnsignedReg::Xmm4 => panic!("No 32-bit XMM register"),
                UnsignedReg::Xmm5 => panic!("No 32-bit XMM register"),
                UnsignedReg::Xmm6 => panic!("No 32-bit XMM register"),
                UnsignedReg::Xmm7 => panic!("No 32-bit XMM register"),
            },
            Size::Qword => match self {
                UnsignedReg::Rax => Reg::Rax,
                UnsignedReg::Rbp => Reg::Rbp,
                UnsignedReg::Rsp => Reg::Rsp,
                UnsignedReg::Rbx => Reg::Rbx,
                UnsignedReg::Rdi => Reg::Rdi,
                UnsignedReg::Rsi => Reg::Rsi,
                UnsignedReg::Rdx => Reg::Rdx,
                UnsignedReg::Rcx => Reg::Rcx,
                UnsignedReg::R8 => Reg::R8,
                UnsignedReg::R9 => Reg::R9,
                UnsignedReg::R10 => Reg::R10,
                UnsignedReg::R11 => Reg::R11,
                UnsignedReg::R12 => Reg::R12,
                UnsignedReg::R13 => Reg::R13,
                UnsignedReg::R14 => Reg::R14,
                UnsignedReg::R15 => Reg::R15,
                UnsignedReg::Xmm0 => Reg::Xmm0,
                UnsignedReg::Xmm1 => Reg::Xmm1,
                UnsignedReg::Xmm2 => Reg::Xmm2,
                UnsignedReg::Xmm3 => Reg::Xmm3,
                UnsignedReg::Xmm4 => Reg::Xmm4,
                UnsignedReg::Xmm5 => Reg::Xmm5,
                UnsignedReg::Xmm6 => Reg::Xmm6,
                UnsignedReg::Xmm7 => Reg::Xmm7,
            },
        }
    }
}
