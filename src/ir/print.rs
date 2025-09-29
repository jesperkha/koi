use crate::ir::Ins;

pub fn print_ir(ir: Vec<Ins>) {
    let mut indent = 0;
    for i in ir {
        println!("{}{}", "    ".repeat(indent), i);
        match i {
            Ins::Func(_) => {
                println!("{}{{", "    ".repeat(indent));
                indent += 1;
            }
            Ins::Return(_, _) => {
                indent -= 1;
                println!("{}}}", "    ".repeat(indent));
            }
            _ => {}
        }
    }
}
