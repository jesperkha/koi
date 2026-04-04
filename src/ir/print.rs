use crate::ir::{Decl, IfIns, Ins, Unit};

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

pub fn ins_to_string(unit: &Unit, ins: &Ins, indent: usize) -> String {
    match ins {
        Ins::Store(ins) => {
            format!(
                "${} {} = {}\n",
                ins.const_id,
                unit.types.type_to_string(ins.ty),
                ins.rval
            )
        }
        Ins::Assign(ins) => {
            format!(
                "{} {} = {}\n",
                ins.lval,
                unit.types.type_to_string(ins.ty),
                ins.rval
            )
        }
        Ins::Return(ty, value) => format!("ret {} {}\n", unit.types.type_to_string(*ty), value),
        Ins::Call(call) => {
            format!(
                "{} {} = call {}({})\n",
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
            "${} {} = {} {} {}\n",
            ins.result,
            unit.types.type_to_string(ins.ty),
            ins.op,
            ins.lhs,
            ins.rhs,
        ),
        Ins::Unary(ins) => format!(
            "${} {} = {} {}\n",
            ins.result,
            unit.types.type_to_string(ins.ty),
            ins.op,
            ins.rhs,
        ),
        Ins::Intrinsic(int) => {
            format!(
                "{}intrinsic {}({})\n",
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
        Ins::If(ins) => if_to_string(unit, ins, indent),
    }
}

fn if_to_string(unit: &Unit, ins: &IfIns, indent: usize) -> String {
    format!(
        "if {}\n{}{}",
        ins.cond,
        ins_to_string_indent(unit, &ins.block.ins, indent + 1),
        match &*ins.elseif {
            crate::ir::ElseBlock::ElseIf(ins) => {
                format!(
                    "{}else {}",
                    "    ".repeat(indent),
                    if_to_string(unit, ins, indent)
                )
            }
            crate::ir::ElseBlock::Else(block) => {
                format!(
                    "{}else\n{}",
                    "    ".repeat(indent),
                    ins_to_string_indent(unit, &block.ins, indent + 1)
                )
            }
            crate::ir::ElseBlock::None => "".into(),
        }
    )
}

fn ins_to_string_indent(unit: &Unit, ins: &Vec<Ins>, indent: usize) -> String {
    let mut s = String::new();
    for i in ins {
        let ins = ins_to_string(unit, i, indent);
        s.push_str(format!("{}{}", "    ".repeat(indent), ins).as_str());
    }
    s
}
