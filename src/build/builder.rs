use crate::ir::Ins;

pub trait Builder {
    fn assemble_package(&self, ins: Vec<Ins>);
}
