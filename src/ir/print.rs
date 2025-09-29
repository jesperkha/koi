use crate::ir::Instruction;

pub fn print_ir(ir: Vec<Instruction>) {
    let mut indent = 0;
    for i in ir {
        println!("{}{}", "    ".repeat(indent), i);
        match i {
            Instruction::Func(_) => {
                println!("{}{{", "    ".repeat(indent));
                indent += 1;
            }
            Instruction::Return(_, _) => {
                indent -= 1;
                println!("{}}}", "    ".repeat(indent));
            }
            _ => {}
        }
    }
}
