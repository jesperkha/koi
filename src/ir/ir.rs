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

pub enum LValue {
    Const(ConstId),
    Param(usize),
}

pub enum Ins {
    Store(StoreIns),
    Assign(AssignIns),
    Return(Type, Value),
    Func(FuncInst),
    Extern(ExternFuncInst),
    Call(CallIns),
    StringData(StringDataIns),
}

pub struct StoreIns {
    pub id: ConstId,
    pub ty: Type,
    pub value: Value,
}

pub struct AssignIns {
    pub lval: LValue,
    pub ty: Type,
    pub value: Value,
}

pub struct StringDataIns {
    pub name: String,
    pub length: usize,
    pub value: String,
}

pub struct ExternFuncInst {
    pub name: String,
    pub params: Vec<Type>,
    pub ret: Type,
}

pub struct FuncInst {
    pub name: String,
    pub public: bool,
    pub params: Vec<Type>,
    pub ret: Type,
    pub body: Vec<Ins>,
    pub stacksize: usize,
}

pub struct CallIns {
    pub callee: Value,
    pub ty: Type,
    pub args: Vec<(Type, Value)>,
    pub result: ConstId,
}

pub enum Value {
    Void,
    Float(f64),
    Int(i64),
    Const(ConstId),
    Param(usize),
    Function(String),
    Data(String),
}

#[derive(Debug)]
pub enum Type {
    Primitive(Primitive),
    Ptr(Box<Type>),
    Object(String, Vec<Type>, usize), // List of fields and total size (not aligned)
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
    Str,
}

pub trait IRVisitor<T> {
    fn visit_func(&mut self, f: &FuncInst) -> T;
    fn visit_extern(&mut self, f: &ExternFuncInst) -> T;
    fn visit_call(&mut self, c: &CallIns) -> T;
    fn visit_static_string(&mut self, d: &StringDataIns) -> T;
    fn visit_ret(&mut self, ty: &Type, v: &Value) -> T;
    fn visit_store(&mut self, ins: &StoreIns) -> T;
    fn visit_assign(&mut self, ins: &AssignIns) -> T;
}

impl Ins {
    pub fn accept<T>(&self, v: &mut dyn IRVisitor<T>) -> T {
        match self {
            Ins::Store(ins) => v.visit_store(ins),
            Ins::Return(ty, value) => v.visit_ret(ty, value),
            Ins::Func(func) => v.visit_func(func),
            Ins::Extern(func) => v.visit_extern(func),
            Ins::Call(call) => v.visit_call(call),
            Ins::StringData(data) => v.visit_static_string(data),
            Ins::Assign(ins) => v.visit_assign(ins),
        }
    }
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
                Primitive::F64 | Primitive::U64 | Primitive::I64 | Primitive::Str => 8,
            },
            Type::Object(_, _, size) => *size,
            Type::Ptr(_) => 8,
        }
    }
}

impl fmt::Display for Ins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ins::Store(ins) => write!(f, "${} {} = {}", ins.id, ins.ty, ins.value),
            Ins::Assign(ins) => write!(f, "{} {} = {}", ins.lval, ins.ty, ins.value),
            Ins::Return(ty, value) => write!(f, "ret {} {}", ty, value),
            Ins::Extern(func) => {
                write!(
                    f,
                    "extern func {}({}) {}\n",
                    func.name,
                    func.params
                        .iter()
                        .map(Type::to_string)
                        .collect::<Vec<String>>()
                        .join(", "),
                    func.ret,
                )
            }
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
            Ins::Call(call) => {
                write!(
                    f,
                    "${} {} = call {}({})",
                    call.result,
                    call.ty,
                    call.callee,
                    call.args
                        .iter()
                        .map(|a| format!("{} {}", a.0, a.1))
                        .collect::<Vec<String>>()
                        .join(", "),
                )
            }
            Ins::StringData(data) => write!(f, "string .{} = \"{}\"", data.name, data.value),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{}", format!("{:?}", p).to_lowercase()),
            Type::Object(name, _, _) => write!(f, "{}", name),
            Type::Ptr(t) => write!(f, "ptr<{}>", t),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => Ok(()),
            Value::Int(s) => write!(f, "{}", s),
            Value::Float(s) => write!(f, "{}", s),
            Value::Const(s) => write!(f, "${}", s),
            Value::Param(s) => write!(f, "%{}", s),
            Value::Function(s) => write!(f, "{}", s),
            Value::Data(s) => write!(f, ".{}", s),
        }
    }
}

impl fmt::Display for LValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LValue::Const(s) => write!(f, "${}", s),
            LValue::Param(s) => write!(f, "%{}", s),
        }
    }
}
