use crate::{build::TransUnit, config::Config, ir::IRUnit};

pub trait Builder<'a>: Sized {
    fn new(config: &'a Config) -> Self;
    fn assemble(self, unit: IRUnit) -> Result<TransUnit, String>;
}

pub fn assemble<'a, B: Builder<'a>>(config: &'a Config, unit: IRUnit) -> Result<TransUnit, String> {
    B::new(config).assemble(unit)
}
