use std::fmt::Display;

pub enum AssemblyIns {
    Mov(Destination, Source, Size),
    Lea(Register, Label),
    Ret,
}

pub enum Size {
    Byte,
    Word,
    Dword,
    Qword,
}

pub enum Source {
    Register(Register),
    StackOffset(StackOffset),
    Label(Label),
}

pub enum Destination {
    Register(Register),
    StackOffset(StackOffset),
}

pub enum Register {
    Rax,

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

impl Display for AssemblyIns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblyIns::Mov(dst, src, size) => write!(f, "mov {} {}, {}", dst, src, size),
            AssemblyIns::Lea(register, label) => write!(f, "lea {}, {}", register, label),
            AssemblyIns::Ret => write!(f, "ret"),
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

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Register(reg) => write!(f, "{}", reg),
            Source::StackOffset(stack) => write!(f, "{}", stack),
            Source::Label(label) => write!(f, "{}", label.name),
        }
    }
}

impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Destination::Register(reg) => write!(f, "{}", reg),
            Destination::StackOffset(stack) => write!(f, "{}", stack),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Register::Rax => "rax",
                Register::Rdi => "rdi",
                Register::Rsi => "rsi",
                Register::Rdx => "rdx",
                Register::Rcx => "rcx",
                Register::R8 => "r8",
                Register::R9 => "r9",
                Register::Xmm0 => "xmm0",
                Register::Xmm1 => "xmm1",
                Register::Xmm2 => "xmm2",
                Register::Xmm3 => "xmm3",
                Register::Xmm4 => "xmm4",
                Register::Xmm5 => "xmm5",
                Register::Xmm6 => "xmm6",
                Register::Xmm7 => "xmm7",
            }
        )
    }
}
