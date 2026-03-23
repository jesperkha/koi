use crate::ir::{Decl, Ins, Unit};

pub fn print_ir(unit: &Unit) {
    println!("{}", unit_to_string(unit));
}

pub fn unit_to_string(unit: &Unit) -> String {
    let mut s = String::new();

    for decl in &unit.decls {
        match decl {
            Decl::Extern(func) => {
                s += &format!(
                    "extern func {}({}) {}\n",
                    func.name,
                    func.params
                        .iter()
                        .map(|ty| unit.types.type_to_string(*ty))
                        .collect::<Vec<String>>()
                        .join(", "),
                    unit.types.type_to_string(func.ret),
                );
            }
            Decl::Func(func) => {
                s += &format!(
                    "func {}({}) {}\n",
                    func.name,
                    func.params
                        .iter()
                        .map(|ty| unit.types.type_to_string(*ty))
                        .collect::<Vec<_>>()
                        .join(", "),
                    unit.types.type_to_string(func.ret),
                );
                s += &ins_to_string_indent(unit, &func.body.ins, 1);
            }
        }
    }

    s
}

pub fn ins_to_string(unit: &Unit, ins: &Ins) -> String {
    match ins {
        Ins::Store(ins) => {
            format!(
                "${} {} = {}",
                ins.const_id,
                unit.types.type_to_string(ins.ty),
                ins.rval
            )
        }
        Ins::Assign(ins) => {
            format!(
                "{} {} = {}",
                ins.lval,
                unit.types.type_to_string(ins.ty),
                ins.rval
            )
        }
        Ins::Return(ty, value) => format!("ret {} {}", unit.types.type_to_string(*ty), value),
        Ins::Call(call) => {
            format!(
                "{} {} = call {}({})",
                call.result,
                unit.types.type_to_string(call.ty),
                call.callee,
                call.args
                    .iter()
                    .map(|a| format!("{} {}", a.1, unit.types.type_to_string(a.0)))
                    .collect::<Vec<String>>()
                    .join(", "),
            )
        }
        Ins::Intrinsic(int) => {
            format!(
                "{}intrinsic {}({})",
                int.result.as_ref().map_or("".into(), |dest| format!(
                    "{} {} = ",
                    dest,
                    unit.types.type_to_string(int.ty)
                )),
                int.kind,
                int.args
                    .iter()
                    .map(|a| format!("{} {}", a.1, unit.types.type_to_string(a.0)))
                    .collect::<Vec<String>>()
                    .join(", "),
            )
        }
    }
}

fn ins_to_string_indent(unit: &Unit, ins: &Vec<Ins>, indent: usize) -> String {
    let mut s = String::new();
    for i in ins {
        let ins = ins_to_string(unit, i);
        s.push_str(format!("{}{}\n", "    ".repeat(indent), ins).as_str());
    }

    if indent > 0 {
        s += "\n";
    }

    s
}
