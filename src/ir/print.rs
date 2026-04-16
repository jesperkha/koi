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
                s += "\n";
            }
        }
    }

    s
}

pub fn ins_to_string_oneline(unit: &Unit, ins: &Ins) -> String {
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
        Ins::Binary(ins) => format!(
            "${} {} = {} {} {}",
            ins.result,
            unit.types.type_to_string(ins.ty),
            ins.op,
            ins.lhs,
            ins.rhs,
        ),
        Ins::Unary(ins) => format!(
            "${} {} = {} {}",
            ins.result,
            unit.types.type_to_string(ins.ty),
            ins.op,
            ins.rhs,
        ),
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
        Ins::If(if_ins) => format!("if {}", if_ins.cond),
        Ins::While(if_ins) => format!("while {}", if_ins.cond),
        Ins::Break => "break".into(),
        Ins::Continue => "continue".into(),
        Ins::Conditional(ins) => {
            format!("${} = cond {} {} {}", ins.result, ins.lhs, ins.op, ins.rhs)
        }
    }
}

fn ins_to_string(unit: &Unit, ins: &Ins, indent: usize) -> String {
    match ins {
        Ins::If(ins) => format!(
            "if {}\n{}{}{}",
            ins.cond,
            ins_to_string_indent(unit, &ins.block.ins, indent + 1),
            ins.elseif
                .iter()
                .map(|elseif| {
                    format!(
                        "{}else if (\n{}{}): {}\n{}",
                        "    ".repeat(indent),
                        ins_to_string_indent(unit, &elseif.cond_ins, indent + 1),
                        "    ".repeat(indent),
                        elseif.cond,
                        ins_to_string_indent(unit, &elseif.block.ins, indent + 1)
                    )
                })
                .collect::<Vec<String>>()
                .join(""),
            ins.elseblock.as_ref().map_or("".into(), |block| {
                format!(
                    "{}else\n{}",
                    "    ".repeat(indent),
                    ins_to_string_indent(unit, &block.ins, indent + 1)
                )
            })
        ),
        Ins::While(ins) => format!(
            "while {}\n{}",
            ins.cond,
            ins_to_string_indent(unit, &ins.block.ins, indent + 1),
        ),
        Ins::Conditional(ins) => format!(
            "${} = cond {} {} {}\n{}\n{}",
            ins.result,
            ins.lhs,
            ins.op,
            ins.rhs,
            ins_to_string_indent(unit, &ins.lhs_ins, indent + 1),
            ins_to_string_indent(unit, &ins.rhs_ins, indent + 1)
        ),
        // TODO: print cond ins
        _ => ins_to_string_oneline(unit, ins),
    }
}

fn ins_to_string_indent(unit: &Unit, ins: &Vec<Ins>, indent: usize) -> String {
    let mut s = String::new();
    for i in ins {
        let ins = ins_to_string(unit, i, indent);
        s.push_str(format!("{}{}\n", "    ".repeat(indent), ins).as_str());
    }
    s
}
