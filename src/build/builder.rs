use crate::{build::TransUnit, ir::IRUnit};

pub trait Builder {
    fn assemble(self, unit: IRUnit) -> Result<TransUnit, String>;
}
