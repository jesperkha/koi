use core::fmt;

pub type ConstId = usize;

pub enum Ins {
    Store(ConstId, Type, Value),
    Return(Type, Value),
    Func(FuncInst),
}

pub struct FuncInst {
    pub name: String,
    pub params: Vec<Type>,
    pub ret: Type,
    pub body: Vec<Ins>,
}

pub enum Value {
    Void,
    Str(String),
    Float(f64),
    Int(i64),
    Const(ConstId),
    Param(usize),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Type {
    Primitive(Primitive),
    Object(String, Vec<Type>, usize), // List of fields and total size (not aligned)
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
            Type::Object(_, _, size) => *size,
        }
    }
}

impl fmt::Display for Ins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ins::Store(var, ty, value) => write!(f, "${} {} = {}", var, ty, value),
            Ins::Return(ty, value) => write!(f, "ret {} {}", ty, value),
            Ins::Func(func) => {
                write!(
                    f,
                    "func {}({}) {}",
                    func.name,
                    func.params
                        .iter()
                        .map(Type::to_string)
                        .collect::<Vec<String>>()
                        .join(", "),
                    func.ret,
                )
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{}", format!("{:?}", p).to_lowercase()),
            Type::Object(name, _, _) => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => Ok(()),
            Value::Str(s) => write!(f, "{}", s),
            Value::Int(s) => write!(f, "{}", s),
            Value::Float(s) => write!(f, "{}", s),
            Value::Const(s) => write!(f, "${}", s),
            Value::Param(s) => write!(f, "%{}", s),
        }
    }
}
