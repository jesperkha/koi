mod assembly;
mod build;
mod reg_alloc;
mod x86;

#[cfg(test)]
mod tests;

pub use assembly::*;
pub use build::assemble;
pub use x86::{BuildConfig, LinkMode, build};
