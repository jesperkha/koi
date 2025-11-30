mod ast;
mod check;
mod checker;
mod context;
mod deps;
mod package;
mod symtable;
mod types;

pub use check::type_check;

pub use ast::*;
pub use deps::*;
pub use types::*;

pub use checker::Checker;
pub use context::TypeContext;
pub use package::Package;
pub use symtable::SymTable;

#[cfg(test)]
mod checker_test;
