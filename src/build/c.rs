use crate::{build::c::emit::emit, config::Config, ir};

mod c;
mod emit;

pub fn build(program: ir::ProgramIR, config: &Config) -> Result<(), String> {
    for unit in program.units {
        let decls = emit(unit, config);

        for decl in decls {
            println!("{}", decl);
        }
    }
    Ok(())
}
