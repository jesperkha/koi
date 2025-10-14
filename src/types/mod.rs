mod checker;
mod context;
mod pkg;
mod symtable;
mod types;

pub use checker::check;
pub use context::TypeContext;
pub use pkg::Package;
pub use symtable::SymTable;
pub use types::*;

#[cfg(test)]
mod checker_test;
