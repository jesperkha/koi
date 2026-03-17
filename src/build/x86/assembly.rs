use std::fmt::Display;

pub struct File {
    pub data_section: Vec<DataDecl>,
    pub text_section: Vec<TextDecl>,
}

pub enum DataDecl {
    String { label: String, content: String },
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
    Push(Src),
    Sub(Dest, Src),
    Mov(Dest, Src),
    Lea(Reg, Label),
    Call(Label),
    Leave,
    Ret,
}

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

pub enum Immediate {
    Int(i64),
    Uint(u64),
    Float(f64),
}

pub enum Dest {
    Reg(Reg),
    StackOffset(StackOffset),
}

pub enum Reg {
    Rax,

    Rbp,
    Rsp,

    // Integer parameters
    Rdi,
    Rsi,
    Rdx,
    Rcx,
    R8,
    R9,

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

pub struct StackOffset {
    pub offset: usize,
    pub size: Size,
}

pub struct Label {
    pub name: String,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ".intel_syntax noprefix\n")?;

        write!(f, ".section .data\n\n")?;
        for decl in &self.data_section {
            write!(f, "{}\n", decl)?;
        }

        write!(f, ".section .text\n\n")?;
        for decl in &self.text_section {
            write!(f, "{}\n", decl)?;
        }

        write!(f, ".section .note.GNU-stack,\"\",@progbits\n")?;
        Ok(())
    }
}

impl Display for DataDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataDecl::String { label, content } => {
                write!(f, ".{}: .asciz \"{}\"", label, content)
            }
        }
    }
}

impl Display for TextDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextDecl::Function { global, name, ins } => {
                if *global {
                    write!(f, ".globl {}\n", name)?;
                }
                write!(f, "{}:\n", name)?;
                for i in ins {
                    write!(f, "    {}\n", i)?;
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
            Asm::Mov(dst, src) => write!(f, "mov {}, {}", dst, src),
            Asm::Lea(register, label) => write!(f, "lea {}, {}", register, label),
            Asm::Sub(dst, src) => write!(f, "sub {}, {}", dst, src),
            Asm::Push(source) => write!(f, "push {}", source),
            Asm::Leave => write!(f, "leave"),
            Asm::Ret => write!(f, "ret"),
            Asm::Call(label) => write!(f, "call {}", label),
        }
    }
}

impl Display for StackOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} PTR [rbp-{}]", self.size, self.offset)
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Size::Byte => "BYTE",
                Size::Word => "WORD",
                Size::Dword => "DWORD",
                Size::Qword => "QWORD",
            }
        )
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
            Src::Label(label) => write!(f, "{}", label.name),
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
        write!(
            f,
            "{}",
            match self {
                Reg::Rax => "rax",
                Reg::Rdi => "rdi",
                Reg::Rsi => "rsi",
                Reg::Rdx => "rdx",
                Reg::Rcx => "rcx",
                Reg::R8 => "r8",
                Reg::R9 => "r9",
                Reg::Xmm0 => "xmm0",
                Reg::Xmm1 => "xmm1",
                Reg::Xmm2 => "xmm2",
                Reg::Xmm3 => "xmm3",
                Reg::Xmm4 => "xmm4",
                Reg::Xmm5 => "xmm5",
                Reg::Xmm6 => "xmm6",
                Reg::Xmm7 => "xmm7",
                Reg::Rbp => "rbp",
                Reg::Rsp => "rsp",
            }
        )
    }
}
