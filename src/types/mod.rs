mod checker;
mod context;
mod exports;
mod namespace;
mod package;
mod symtable;
mod types;

pub use checker::check_fileset;
pub use context::TypeContext;
pub use exports::Exports;
pub use namespace::Namespace;
pub use package::Package;
pub use symtable::SymTable;
pub use types::*;

#[cfg(test)]
mod checker_test;
