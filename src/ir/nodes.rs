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
pub type ParamId = usize;

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
    Param(ParamId),
}

pub enum Ins {
    Store(StoreIns),
    Assign(AssignIns),
    Call(CallIns),
    Intrinsic(IntrinsicIns),
    Return(IRTypeId, RValue),
    Binary(BinaryIns),
    Unary(UnaryIns),
    If(IfIns),
    While(WhileIns),
}

pub enum IRBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    And,
    Or,
}

pub enum IRUnaryOp {
    Neg,
    Not,
}

pub struct BinaryIns {
    pub ty: IRTypeId,
    pub op: IRBinaryOp,
    pub lhs: RValue,
    pub rhs: RValue,
    pub result: ConstId,
}

pub struct UnaryIns {
    pub ty: IRTypeId,
    pub op: IRUnaryOp,
    pub rhs: RValue,
    pub result: ConstId,
}

pub struct StoreIns {
    /// Type of the value being stored
    pub ty: IRTypeId,
    /// Destination value id being assigned to
    pub const_id: ConstId,
    /// Value being assigned
    pub rval: RValue,
}

pub struct AssignIns {
    /// Type of the value being stored
    pub ty: IRTypeId,
    /// Destination value being assigned to
    pub lval: LValue,
    /// Value being assigned
    pub rval: RValue,
}

pub struct IfIns {
    /// Boolean condition
    pub cond: RValue,
    /// Then-block
    pub block: Block,
    /// List of else-if branches, or empty list
    pub elseif: Vec<ElseIf>,
    /// Optional final else block
    pub elseblock: Option<Block>,
}

pub struct ElseIf {
    /// List of instructions to calculate the condition
    pub cond_ins: Vec<Ins>,
    /// Boolean condition
    pub cond: RValue,
    /// Then-block
    pub block: Block,
}

pub struct WhileIns {
    /// Boolean condition
    pub cond: RValue,
    /// Then-block
    pub block: Block,
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

impl fmt::Display for IRBinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IRBinaryOp::Add => "add",
            IRBinaryOp::Sub => "sub",
            IRBinaryOp::Mul => "mul",
            IRBinaryOp::Div => "div",
            IRBinaryOp::Mod => "mod",
            IRBinaryOp::Eq => "eq",
            IRBinaryOp::Ne => "ne",
            IRBinaryOp::Gt => "gt",
            IRBinaryOp::Ge => "ge",
            IRBinaryOp::Lt => "lt",
            IRBinaryOp::Le => "le",
            IRBinaryOp::And => "and",
            IRBinaryOp::Or => "or",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for IRUnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IRUnaryOp::Neg => "neg",
            IRUnaryOp::Not => "not",
        };
        write!(f, "{}", s)
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
