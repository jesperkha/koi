use std::fmt::Display;

pub struct Ast {
    pub decls: Vec<Decl>,
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.decls
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("\n\n")
        )
    }
}

pub enum Decl {
    Include(String),
    Function {
        name: String,
        params: Vec<(usize, Type)>,
        ret: Type,
        body: Vec<Stmt>,
    },
}

pub enum Type {
    Void,

    Int8,
    Int16,
    Int32,
    Int64,

    Uint8,
    Uint16,
    Uint32,
    Uint64,

    Float,

    Pointer(Box<Type>),
}

pub enum BinaryOp {
    Plus,
    Minus,
    Div,
    Mult,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Modulo,
}

pub enum UnaryOp {
    Minus,
    Not,
}

pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    UintLit(u64),
    VarLit(usize),
}

pub enum Stmt {
    Return(Option<Expr>),
    Binary {
        ty: Type,
        result: usize,
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        ty: Type,
        result: usize,
        op: UnaryOp,
        expr: Box<Expr>,
    },
    VarDecl {
        ty: Type,
        id: usize,
        value: Box<Expr>,
    },
    VarAssign {
        lhs: usize,
        rhs: Box<Expr>,
    },
    Call {
        ty: Type,
        dest: usize,
        callee: String,
        args: Vec<Expr>,
    },
    Break,
    Continue,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Int8 => write!(f, "int8_t"),
            Type::Int16 => write!(f, "int16_t"),
            Type::Int32 => write!(f, "int32_t"),
            Type::Int64 => write!(f, "int64_t"),
            Type::Uint8 => write!(f, "uint8_t"),
            Type::Uint16 => write!(f, "uint16_t"),
            Type::Uint32 => write!(f, "uint32_t"),
            Type::Uint64 => write!(f, "uint64_t"),
            Type::Float => write!(f, "float"),
            Type::Pointer(t) => write!(f, "{}*", t),
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryOp::Minus => "-",
                UnaryOp::Not => "!",
            }
        )
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinaryOp::Plus => "+",
                BinaryOp::Minus => "-",
                BinaryOp::Div => "/",
                BinaryOp::Mult => "*",
                BinaryOp::Equal => "==",
                BinaryOp::NotEqual => "!=",
                BinaryOp::Greater => ">",
                BinaryOp::Less => "<",
                BinaryOp::GreaterEqual => ">=",
                BinaryOp::LessEqual => "<=",
                BinaryOp::Modulo => "%",
            }
        )
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::IntLit(i) => write!(f, "{i}"),
            Expr::FloatLit(i) => write!(f, "{i}"),
            Expr::UintLit(u) => write!(f, "{u}"),
            Expr::VarLit(id) => write!(f, "t{id}"),
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Return(expr) => {
                write!(
                    f,
                    "return {};",
                    expr.as_ref().map_or("".to_string(), |e| e.to_string())
                )
            }
            Stmt::Binary {
                ty,
                result,
                op,
                left,
                right,
            } => write!(f, "{ty} t{result} = {left} {op} {right};"),
            Stmt::Unary {
                ty,
                result,
                op,
                expr,
            } => write!(f, "{ty} t{result} = {op}{expr};"),
            Stmt::VarDecl { ty, id, value } => write!(f, "{ty} t{id} = {value};"),
            Stmt::VarAssign { lhs, rhs } => write!(f, "t{lhs} = {rhs};"),
            Stmt::Call {
                ty,
                callee,
                dest,
                args,
            } => write!(
                f,
                "{ty} t{dest} = {callee}({});",
                args.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Stmt::Break => write!(f, "break;"),
            Stmt::Continue => write!(f, "continue;"),
        }
    }
}

impl Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decl::Function {
                name,
                params,
                ret,
                body,
            } => {
                write!(
                    f,
                    "{ret} {name}({}) {{\n{}\n}}",
                    params
                        .iter()
                        .map(|(id, ty)| { format!("{} t{}", ty, id) })
                        .collect::<Vec<_>>()
                        .join(", "),
                    body.iter()
                        .map(|stmt| stmt.to_string())
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            }
            Decl::Include(path) => write!(f, "#include \"{path}\""),
        }
    }
}
