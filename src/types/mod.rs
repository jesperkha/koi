mod ast;
mod check;
mod checker;
mod context;
mod symtable;
mod tests;
mod types;

pub use check::type_check;

pub use ast::*;
pub use types::*;

pub use checker::Checker;
pub use context::TypeContext;
pub use symtable::VarTable;
