use crate::{build::builder, ir::IRVisitor};

pub struct Builder {}

impl builder::Builder for Builder {
    fn assemble_package(&self, ins: Vec<crate::ir::Ins>) {
        todo!()
    }
}

impl IRVisitor<()> for Builder {
    fn visit_func(&self, f: &crate::ir::FuncInst) {
        todo!()
    }

    fn visit_ret(&self, ty: &crate::ir::Type, v: &crate::ir::Value) {
        todo!()
    }

    fn visit_store(&self, id: crate::ir::ConstId, ty: &crate::ir::Type, v: &crate::ir::Value) {
        todo!()
    }
}
