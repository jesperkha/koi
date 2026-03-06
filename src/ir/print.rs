use crate::ir::{Decl, Ins, Unit};

pub fn print_ir(unit: &Unit) {
    println!("{}", ir_to_string(unit));
}

pub fn ir_to_string(unit: &Unit) -> String {
    let mut s = String::new();

    s += &unit.types.dump();
    s += "\n\n";

    for decl in &unit.decls {
        match decl {
            Decl::Extern(_) => s += &decl.to_string(),
            Decl::Func(d) => {
                s += &decl.to_string();
                s += "\n";
                s += &ins_to_string_indent(&d.body.ins, 1);
            }
        }
        s += "\n";
    }

    s
}

fn ins_to_string_indent(ins: &Vec<Ins>, indent: usize) -> String {
    let mut s = String::new();
    for i in ins {
        s.push_str(format!("{}{}\n", "    ".repeat(indent), i).as_str());
    }

    if indent > 0 {
        s += "\n";
    }

    s
}
