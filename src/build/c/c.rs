use std::fmt::Display;

use crate::ir::{self, IRType};

pub enum Node {
    Function(FunctionNode),
    Block(BlockNode),
}

pub enum CType {
    Void,
    Pointer(Box<CType>),
    Named(String),
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

pub struct TypeModifier {
    pub mstatic: bool,
    pub mconst: bool,
    pub unsigned: bool,
}

pub struct FunctionNode {
    pub args: Vec<CType>,
    pub body: BlockNode,
    pub ret: CType,
}

pub struct BlockNode {
    pub nodes: Vec<Node>,
}

impl Display for CType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CType::Pointer(ctype) => &format!("*{}", ctype),
            CType::Named(name) => name,
            CType::Void => "void",
            CType::U8 => "uint8_t",
            CType::U16 => "uint16_t",
            CType::U32 => "uint32_t",
            CType::U64 => "uint64_t",
            CType::I8 => "int8_t",
            CType::I16 => "int16_t",
            CType::I32 => "int32_t",
            CType::I64 => "int64_t",
            CType::F32 => "float",
            CType::F64 => "float",
        };

        write!(f, "{}", s)
    }
}

impl From<IRType> for CType {
    fn from(value: IRType) -> Self {
        match value {
            IRType::Primitive(primitive) => match primitive {
                ir::Primitive::Void => Self::Void,
                ir::Primitive::F32 => Self::F32,
                ir::Primitive::F64 => Self::F64,
                ir::Primitive::U8 => Self::U8,
                ir::Primitive::U16 => Self::U16,
                ir::Primitive::U32 => Self::U32,
                ir::Primitive::U64 => Self::U64,
                ir::Primitive::I8 => Self::I8,
                ir::Primitive::I16 => Self::I16,
                ir::Primitive::I32 => Self::I32,
                ir::Primitive::I64 => Self::I64,
                ir::Primitive::String => Self::Pointer(Box::new(Self::U8)),
            },
            IRType::Function(irtypes, irtype) => todo!(),
        }
    }
}

impl Display for TypeModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if self.mstatic {
            s += "static ";
        }
        if self.mconst {
            s += "const ";
        }
        if self.unsigned {
            s += "unsigned ";
        }
        write!(f, "{s}")
    }
}
