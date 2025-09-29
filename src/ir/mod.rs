mod emit;
mod ir;
mod print;

pub use emit::IR;
pub use ir::*;
pub use print::print_ir;

#[cfg(test)]
mod ir_test;
