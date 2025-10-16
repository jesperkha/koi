use crate::ir::{Primitive, Type};

#[derive(Debug)]
pub struct RegAllocator {
    int_regs: Vec<&'static str>,
    float_regs: Vec<&'static str>,
    int_ret: &'static str,
    float_ret: &'static str,

    next_int: usize,
    next_float: usize,
    param_int: usize,
    param_float: usize,
}

impl RegAllocator {
    pub fn new() -> Self {
        Self {
            int_regs: vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"],
            float_regs: vec![
                "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
            ],
            int_ret: "rax",
            float_ret: "xmm0",
            next_int: 0,
            next_float: 0,
            param_int: 0,
            param_float: 0,
        }
    }

    fn is_float(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Primitive(Primitive::F32) | Type::Primitive(Primitive::F64)
        )
    }

    fn is_intlike(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Primitive(
                Primitive::U8
                    | Primitive::U16
                    | Primitive::U32
                    | Primitive::U64
                    | Primitive::I8
                    | Primitive::I16
                    | Primitive::I32
                    | Primitive::I64
                    | Primitive::String
                    | Primitive::Uintptr(_)
            )
        )
    }

    /// Get next available register suitable for given type
    pub fn next_reg(&mut self, ty: &Type) -> String {
        if Self::is_float(ty) {
            assert!(self.next_float < self.float_regs.len());
            let reg = self.float_regs[self.next_float];
            self.next_float += 1;
            reg.to_string()
        } else if Self::is_intlike(ty) {
            assert!(self.next_int < self.int_regs.len());
            let reg = self.int_regs[self.next_int];
            self.next_int += 1;
            reg.to_string()
        } else {
            panic!("unknown type");
        }
    }

    /// Get return register for given type
    pub fn return_reg(&self, ty: &Type) -> String {
        if Self::is_float(ty) {
            self.float_ret.to_string()
        } else if Self::is_intlike(ty) {
            self.int_ret.to_string()
        } else {
            "rax".to_string()
        }
    }

    /// Get next available parameter register for given type
    pub fn next_param_reg(&mut self, ty: &Type) -> String {
        if Self::is_float(ty) {
            assert!(self.param_float < self.float_regs.len());
            let reg = self.float_regs[self.param_float];
            self.param_float += 1;
            reg.to_string()
        } else if Self::is_intlike(ty) {
            assert!(self.param_int < self.int_regs.len());
            let reg = self.int_regs[self.param_int];
            self.param_int += 1;
            reg.to_string()
        } else {
            panic!("unknown type");
        }
    }

    /// Reset all register allocation counters (for new functions or contexts)
    pub fn reset(&mut self) {
        self.next_int = 0;
        self.next_float = 0;
        self.param_int = 0;
        self.param_float = 0;
    }

    /// Reset only parameter registers (for next function call)
    pub fn reset_params(&mut self) {
        self.param_int = 0;
        self.param_float = 0;
    }

    /// Reset only temporary registers (for new expression evaluation)
    pub fn reset_temps(&mut self) {
        self.next_int = 0;
        self.next_float = 0;
    }
}
