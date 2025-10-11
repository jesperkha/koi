use core::fmt;

use crate::ir::print::ir_to_string;

pub struct IRUnit {
    pub ins: Vec<Ins>,
}

impl IRUnit {
    pub fn new(ins: Vec<Ins>) -> Self {
        Self { ins }
    }
}

impl fmt::Display for IRUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ir_to_string(&self.ins))
    }
}

pub type ConstId = usize;

pub enum Ins {
    Package(String),
    Store(ConstId, Type, Value),
    Return(Type, Value),
    Func(FuncInst),
}

pub struct FuncInst {
    pub name: String,
    pub public: bool,
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

pub trait IRVisitor<T> {
    fn visit_package(&mut self, name: &str) -> T;
    fn visit_func(&mut self, f: &FuncInst) -> T;
    fn visit_ret(&mut self, ty: &Type, v: &Value) -> T;
    fn visit_store(&mut self, id: ConstId, ty: &Type, v: &Value) -> T;
}

impl Ins {
    pub fn accept<T>(&self, v: &mut dyn IRVisitor<T>) -> T {
        match self {
            Ins::Package(name) => v.visit_package(&name),
            Ins::Store(id, ty, value) => v.visit_store(*id, ty, value),
            Ins::Return(ty, value) => v.visit_ret(ty, value),
            Ins::Func(func) => v.visit_func(func),
        }
    }
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
            Ins::Package(name) => write!(f, "pkg '{}'", name),
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
