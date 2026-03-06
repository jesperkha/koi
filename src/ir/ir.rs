use core::fmt;

use crate::ir::{IRTypeId, IRTypeInterner};

pub struct ProgramIR {
    pub units: Vec<Unit>,
}

pub struct Unit {
    /// Type mappings for this unit
    pub types: IRTypeInterner,
    /// Module path of this unit in underscore form
    pub name: String,
    /// Declarations in this unit
    pub decls: Vec<Decl>,
    /// Data segments used in this unit
    pub data: Vec<Data>,
}

/// Unique ID of a constant value
pub type ConstId = usize;

/// Index into the units data map
pub type DataIndex = usize;

pub enum Data {
    String(String),
}

pub enum Decl {
    Extern(ExternDecl),
    Func(FuncDecl),
}

pub struct ExternDecl {
    /// The symbol name
    pub name: String,
    /// Parameter types
    pub params: Vec<IRTypeId>,
    /// Return type
    pub ret: IRTypeId,
}

pub struct FuncDecl {
    /// Is this function public (outside of comp unit)
    pub public: bool,
    /// Name of function
    pub name: String,
    /// Parameter types
    pub params: Vec<IRTypeId>,
    /// Return type
    pub ret: IRTypeId,
    /// Function body instructions
    pub body: Block,
    /// Accumulated minimum stack size of body variables
    pub stacksize: usize,
}

pub struct Block {
    pub ins: Vec<Ins>,
}

pub enum LValue {
    Const(ConstId),
    Param(usize),
}

pub enum Ins {
    Store(StoreIns),
    Assign(StoreIns),
    Call(CallIns),
    Intrinsic(IntrinsicIns),
    Return(IRTypeId, RValue),
}

pub struct StoreIns {
    /// Type of the value being stored
    pub ty: IRTypeId,
    /// Destination value being assigned to
    pub lval: LValue,
    /// Value being assigned
    pub rval: RValue,
}

pub struct CallIns {
    /// Return type of the call
    pub ty: IRTypeId,
    /// The callee (should be Function or Const)
    pub callee: RValue,
    /// The arguments of the function call and their types
    pub args: Vec<(IRTypeId, RValue)>,
    /// Destination value being assigned to
    pub result: LValue,
}

pub enum IntrinsicKind {
    Exit,
}

pub struct IntrinsicIns {
    pub kind: IntrinsicKind,
    pub ty: IRTypeId,
    pub args: Vec<(IRTypeId, RValue)>,
    pub result: Option<LValue>,
}

pub enum RValue {
    Void,
    Float(f64),
    Int(i64),
    Uint(u64),
    Const(ConstId),
    Param(usize),
    Function(String),
    Data(DataIndex),
}

impl fmt::Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decl::Extern(func) => {
                write!(
                    f,
                    "extern func {}({}) {}\n",
                    func.name,
                    func.params
                        .iter()
                        .map(|ty| ty.to_string())
                        .collect::<Vec<String>>()
                        .join(", "),
                    func.ret,
                )
            }
            Decl::Func(func) => {
                write!(
                    f,
                    "func {}({}) <{}>",
                    func.name,
                    func.params
                        .iter()
                        .map(|ty| format!("<{}>", ty))
                        .collect::<Vec<_>>()
                        .join(", "),
                    func.ret,
                )
            }
        }
    }
}

impl fmt::Display for Ins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ins::Store(ins) | Ins::Assign(ins) => {
                write!(f, "{} {} = {}", ins.lval, ins.ty, ins.rval)
            }
            Ins::Return(ty, value) => write!(f, "ret <{}> {}", ty, value),
            Ins::Call(call) => {
                write!(
                    f,
                    "{}call {}({})",
                    format!("{} <{}> = ", call.result, call.ty),
                    call.callee,
                    call.args
                        .iter()
                        .map(|a| format!("<{}> {}", a.0, a.1))
                        .collect::<Vec<String>>()
                        .join(", "),
                )
            }
            Ins::Intrinsic(int) => {
                write!(
                    f,
                    "{}intrinsic {}({})",
                    int.result
                        .as_ref()
                        .map_or("".into(), |dest| format!("{} <{}> = ", dest, int.ty)),
                    int.kind,
                    int.args
                        .iter()
                        .map(|a| format!("<{}> {}", a.0, a.1))
                        .collect::<Vec<String>>()
                        .join(", "),
                )
            }
        }
    }
}

impl fmt::Display for IntrinsicKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntrinsicKind::Exit => write!(f, "exit"),
        }
    }
}

impl fmt::Display for RValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RValue::Void => Ok(()),
            RValue::Int(s) => write!(f, "{}", s),
            RValue::Uint(s) => write!(f, "{}", s),
            RValue::Float(s) => write!(f, "{}", s),
            RValue::Const(s) => write!(f, "${}", s),
            RValue::Param(s) => write!(f, "%{}", s),
            RValue::Function(s) => write!(f, "{}", s),
            RValue::Data(s) => write!(f, ".{}", s),
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
