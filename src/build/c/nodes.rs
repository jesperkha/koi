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
    ExternFunc {
        name: String,
        params: Vec<Type>,
        ret: Type,
    },
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
    StrLit(String),
    Not(Box<Expr>),
    Cast(Type, Box<Expr>),
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
    If {
        cond: Expr,
        body: Vec<Stmt>,
        else_: Option<Vec<Stmt>>,
    },
    While {
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    Goto(String),
    Label(String),
}

fn ind(level: usize) -> String {
    "    ".repeat(level)
}

fn fmt_stmts(stmts: &[Stmt], level: usize) -> String {
    let mut out = String::new();
    for (i, stmt) in stmts.iter().enumerate() {
        out += &stmt.to_indented(level);
        let is_last = i + 1 == stmts.len();
        if !is_last {
            out += "\n";
            if matches!(stmt, Stmt::If { .. } | Stmt::While { .. }) {
                out += "\n";
            }
        }
    }
    out
}

impl Stmt {
    fn to_indented(&self, level: usize) -> String {
        let i = ind(level);
        match self {
            Stmt::Return(expr) => format!(
                "{}return {};",
                i,
                expr.as_ref().map_or("".to_string(), |e| e.to_string())
            ),
            Stmt::Binary { ty, result, op, left, right } => {
                format!("{i}{ty} t{result} = {left} {op} {right};")
            }
            Stmt::Unary { ty, result, op, expr } => {
                format!("{i}{ty} t{result} = {op}{expr};")
            }
            Stmt::VarDecl { ty, id, value } => format!("{i}{ty} t{id} = {value};"),
            Stmt::VarAssign { lhs, rhs } => format!("{i}t{lhs} = {rhs};"),
            Stmt::Call { ty, dest, callee, args } => {
                let args_str =
                    args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ");
                if matches!(ty, Type::Void) {
                    format!("{i}{callee}({args_str});")
                } else {
                    format!("{i}{ty} t{dest} = {callee}({args_str});")
                }
            }
            Stmt::If { cond, body, else_ } => {
                let body_str = fmt_stmts(body, level + 1);
                let mut out = format!("{i}if ({cond}) {{\n{body_str}\n{i}}}");
                if let Some(else_stmts) = else_ {
                    let else_str = fmt_stmts(else_stmts, level + 1);
                    out += &format!(" else {{\n{else_str}\n{i}}}");
                }
                out
            }
            Stmt::While { body } => {
                let body_str = fmt_stmts(body, level + 1);
                format!("{i}while (1) {{\n{body_str}\n{i}}}")
            }
            Stmt::Break => format!("{i}break;"),
            Stmt::Continue => format!("{i}continue;"),
            Stmt::Goto(label) => format!("{i}goto {label};"),
            Stmt::Label(label) => format!("{label}:"),
        }
    }
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
            Expr::StrLit(s) => {
                write!(f, "\"")?;
                for ch in s.chars() {
                    match ch {
                        '\\' => write!(f, "\\\\")?,
                        '"' => write!(f, "\\\"")?,
                        '\n' => write!(f, "\\n")?,
                        '\r' => write!(f, "\\r")?,
                        '\t' => write!(f, "\\t")?,
                        c => write!(f, "{c}")?,
                    }
                }
                write!(f, "\"")
            }
            Expr::Not(inner) => write!(f, "!{inner}"),
            Expr::Cast(ty, inner) => write!(f, "({ty})({inner})"),
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_indented(0))
    }
}

impl Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decl::Function { name, params, ret, body } => {
                write!(
                    f,
                    "{ret} {name}({}) {{\n{}\n}}",
                    params
                        .iter()
                        .map(|(id, ty)| format!("{} t{}", ty, id))
                        .collect::<Vec<_>>()
                        .join(", "),
                    fmt_stmts(body, 1),
                )
            }
            Decl::ExternFunc { name, params, ret } => write!(
                f,
                "extern {ret} {name}({});",
                params
                    .iter()
                    .enumerate()
                    .map(|(i, ty)| format!("{ty} t{i}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Decl::Include(path) => write!(f, "#include \"{path}\""),
        }
    }
}
