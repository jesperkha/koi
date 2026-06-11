use core::fmt;

use crate::{IRTypeId, IRTypeInterner};

pub struct ProgramIR {
    pub units: Vec<Unit>,
}

pub struct Unit {
    pub types: IRTypeInterner,
    pub name: String,
    pub decls: Vec<Decl>,
    pub data: Vec<Data>,
}

pub type ConstId = usize;
pub type ParamId = usize;
pub type DataIndex = usize;

pub enum Data {
    String(String),
}

pub enum Decl {
    Extern(ExternDecl),
    Func(FuncDecl),
}

pub struct ExternDecl {
    pub name: String,
    pub params: Vec<IRTypeId>,
    pub ret: IRTypeId,
}

pub struct FuncDecl {
    pub public: bool,
    pub name: String,
    pub params: Vec<IRTypeId>,
    pub ret: IRTypeId,
    pub body: Block,
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
    Cast(CastIns),
    If(IfIns),
    While(WhileIns),
    Conditional(CondIns),
    Break,
    Continue,
}

pub enum IRCondOp {
    And,
    Or,
}

pub struct CondIns {
    pub op: IRCondOp,
    pub lhs_ins: Vec<Ins>,
    pub lhs: RValue,
    pub rhs_ins: Vec<Ins>,
    pub rhs: RValue,
    pub result: ConstId,
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

pub struct CastIns {
    pub from_ty: IRTypeId,
    pub to_ty: IRTypeId,
    pub rval: RValue,
    pub result: ConstId,
}

pub struct StoreIns {
    pub ty: IRTypeId,
    pub const_id: ConstId,
    pub rval: RValue,
}

pub struct AssignIns {
    pub ty: IRTypeId,
    pub lval: LValue,
    pub rval: RValue,
}

pub struct IfIns {
    pub cond: RValue,
    pub block: Block,
    pub elseif: Vec<ElseIf>,
    pub elseblock: Option<Block>,
}

pub struct ElseIf {
    pub cond_ins: Vec<Ins>,
    pub cond: RValue,
    pub block: Block,
}

pub struct WhileIns {
    pub cond_ins: Vec<Ins>,
    pub cond: RValue,
    pub block: Block,
    pub post: Option<Vec<Ins>>,
}

pub struct CallIns {
    pub ty: IRTypeId,
    pub callee: RValue,
    pub args: Vec<(IRTypeId, RValue)>,
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
        write!(
            f,
            "{}",
            match self {
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
            }
        )
    }
}

impl fmt::Display for IRCondOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                IRCondOp::And => "and",
                IRCondOp::Or => "or",
            }
        )
    }
}

impl fmt::Display for IRUnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                IRUnaryOp::Neg => "neg",
                IRUnaryOp::Not => "not",
            }
        )
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
