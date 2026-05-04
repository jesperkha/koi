use std::fmt::Display;

use crate::ir::{self, IRType};

pub enum Decl {
    Extern(ExternNode),
    Function(FunctionNode),
    Block(BlockNode),
}

pub enum Stmt {
    Return(Option<Expr>),
}

pub enum Expr {
    Void,
    Int(i64),
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

pub struct ExternNode {
    pub name: String,
    pub args: Vec<CType>,
    pub ret: CType,
}

pub struct FunctionNode {
    pub name: String,
    pub args: Vec<CType>,
    pub body: BlockNode,
    pub ret: CType,
}

pub struct BlockNode {
    pub stmts: Vec<Stmt>,
}

impl Display for CType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CType::Pointer(ctype) => &format!("{}*", ctype),
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

impl From<&IRType> for CType {
    fn from(value: &IRType) -> Self {
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

impl Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Decl::Extern(node) => format!(
                    "extern {} {}({});\n",
                    node.ret,
                    node.name,
                    args_to_string(&node.args)
                ),
                Decl::Function(node) => format!(
                    "{} {}({})\n{}\n",
                    node.ret,
                    node.name,
                    args_to_string(&node.args),
                    block_to_string(&node.body)
                ),
                Decl::Block(node) => block_to_string(node),
            }
        )
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Stmt::Return(expr) => format!(
                    "return {};",
                    expr.as_ref().map_or("".to_owned(), |e| e.to_string())
                ),
            }
        )
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Expr::Void => format!("void"),
                Expr::Int(n) => format!("{n}"),
            }
        )
    }
}

fn args_to_string(args: &[CType]) -> String {
    args.iter()
        .enumerate()
        .map(|(i, arg)| format!("{} a{}", arg.to_string(), i))
        .collect::<Vec<_>>()
        .join(", ")
}

fn block_to_string(block: &BlockNode) -> String {
    format!(
        "{{\n{}\n}}",
        block
            .stmts
            .iter()
            .map(|stmt| stmt.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    )
}
