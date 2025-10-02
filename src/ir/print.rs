use crate::ir::Ins;

pub fn print_ir(ir: Vec<Ins>) {
    println!("{}", ir_to_string(ir));
}

pub fn ir_to_string(ir: Vec<Ins>) -> String {
    ir_to_string_indent(ir, 0)
}

fn ir_to_string_indent(ir: Vec<Ins>, indent: usize) -> String {
    let mut s = String::new();
    for i in ir {
        s.push_str(format!("{}{}\n", "    ".repeat(indent), i).as_str());
        match i {
            Ins::Func(f) => s.push_str(ir_to_string_indent(f.body, indent + 1).as_str()),
            _ => {}
        }
    }

    if indent > 0 {
        s += "\n";
    }

    s
}
