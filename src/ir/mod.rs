mod emit;
mod ir;
mod print;
mod sym;

pub use emit::IR;
pub use ir::*;
pub use print::print_ir;
pub use sym::*;

#[cfg(test)]
mod ir_test;
