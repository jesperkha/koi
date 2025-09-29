pub type ConstId = usize;

pub enum Instruction {
    Store(ConstId, Type, Value),
    Return(Type, Value),
    Func(FuncInst),
}

pub struct FuncInst {
    pub name: String,
    pub args: Vec<Type>,
    pub ret: Type,
}

pub enum Value {
    Str(String),
    Int(i64),
    Uint(u64),
    Bool(bool),
    Const(ConstId),
}

pub enum Primitive {
    Void,
    F32,
    F64,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Uintptr(Box<Type>),
}

pub enum Type {
    Primitive(Primitive),
    Object(Vec<Type>, usize), // List of fields and total size (not aligned)
}

impl Type {
    /// Get size of type in bytes
    pub fn size(&self) -> usize {
        match self {
            Type::Primitive(primitive) => match primitive {
                Primitive::Void => 0,
                Primitive::U8 | Primitive::I8 => 1,
                Primitive::U16 | Primitive::I16 => 2,
                Primitive::F32 | Primitive::I32 | Primitive::U32 => 4,
                Primitive::F64 | Primitive::U64 | Primitive::I64 | Primitive::Uintptr(_) => 8,
            },
            Type::Object(_, size) => *size,
        }
    }
}
