use crate::{
    ast::{File, FileSet},
    build::{Builder, X86Builder},
    config::Config,
    error::{ErrorSet, Res},
    ir::{IRUnit, emit_ir},
    module::{Module, ModuleGraph, ModulePath},
    parser::parse,
    token::{Source, Token, scan},
    types::type_check,
};

pub fn compare_string_lines_or_panic(ina: String, inb: String) {
    let a: Vec<&str> = ina.trim().split('\n').collect();
    let b: Vec<&str> = inb.trim().split('\n').collect();
    assert_eq!(
        a.len(),
        b.len(),
        "number of lines must be equal, got\n{}\nand\n{}",
        ina,
        inb,
    );

    for (i, line) in a.iter().enumerate() {
        assert_eq!(line.trim(), b.get(i).unwrap().trim());
    }
}

pub fn must<T>(res: Result<T, ErrorSet>) -> T {
    res.unwrap_or_else(|err| panic!("unexpected error: {}", err))
}

pub fn scan_string(src: &str) -> Res<Vec<Token>> {
    let src = Source::new_from_string(src);
    let config = Config::test();
    scan(&src, &config)
}

pub fn parse_string(src: &str) -> Res<File> {
    let src = Source::new_from_string(src);
    let config = Config::test();
    scan(&src, &config).and_then(|toks| parse(src, toks, &config))
}

pub fn check_string<'a>(src: &str, mg: &'a mut ModuleGraph) -> Res<&'a Module> {
    let config = Config::test();
    let fs = FileSet::new(ModulePath::new_str("main"), vec![parse_string(src)?]);
    type_check(fs, mg, &config)
}

pub fn emit_string(src: &str) -> Res<IRUnit> {
    let config = Config::test();
    let mut mg = ModuleGraph::new();
    check_string(src, &mut mg).and_then(|pkg| emit_ir(&pkg, &config))
}

pub fn compile_string(src: &str) -> Result<String, String> {
    let config = Config::test();
    emit_string(src)
        .map_err(|err| err.to_string())
        .and_then(|ir| X86Builder::new(&config).assemble(ir))
        .and_then(|unit| Ok(unit.source))
}

pub fn debug_print_all_steps(src: &str) {
    let config = Config::debug();
    let source = Source::new_from_string(src);
    let mut mg = ModuleGraph::new();

    scan(&source, &config)
        .and_then(|toks| parse(source, toks, &config))
        .and_then(|file| {
            println!("SOURCE CODE");
            println!("===========\n");
            println!("{}", file);
            type_check(
                FileSet::new(ModulePath::new_str("main"), vec![file]),
                &mut mg,
                &config,
            )
        })
        .and_then(|pkg| emit_ir(&pkg, &config))
        .map_err(|err| err.to_string())
        .and_then(|unit| {
            println!("INTERMEDIATE REPRESENTATION");
            println!("===========================\n");
            println!("{}", unit);
            X86Builder::new(&config).assemble(unit)
        })
        .map_or_else(
            |err| println!("{}", err),
            |unit| {
                println!("ASSEMBLY (X86)");
                println!("==============\n");
                println!("{}", unit.source);
            },
        );
}
