mod emit;
mod ir;
mod print;
mod sym;

pub use emit::emit_ir;
pub use ir::*;
pub use print::print_ir;
pub use sym::SymTracker;

#[cfg(test)]
mod ir_test;
