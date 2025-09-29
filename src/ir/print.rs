use crate::ir::Ins;

pub fn print_ir(ir: Vec<Ins>) {
    println!("{}", ir_to_string(ir));
}

pub fn ir_to_string(ir: Vec<Ins>) -> String {
    let mut s = String::new();
    let mut indent = 0;
    for i in ir {
        s.push_str(format!("{}{}\n", "    ".repeat(indent), i).as_str());
        match i {
            Ins::Func(_) => {
                s.push_str(format!("{}{{\n", "    ".repeat(indent)).as_str());
                indent += 1;
            }
            Ins::Return(_, _) => {
                indent -= 1;
                s.push_str(format!("{}}}\n", "    ".repeat(indent)).as_str());
            }
            _ => {}
        }
    }

    s
}
