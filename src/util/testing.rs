use crate::{
    ast::{File, Printer},
    build::{Builder, X86Builder},
    config::Config,
    error::{ErrorSet, Res},
    ir::{IRUnit, emit_ir, print_ir},
    parser::parse,
    scanner::scan,
    token::{Source, Token},
    types::{Package, check},
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

pub fn check_string(src: &str) -> Res<Package> {
    let config = Config::test();
    check(vec![parse_string(src)?], &config)
}

pub fn emit_string(src: &str) -> Res<IRUnit> {
    let config = Config::test();
    check_string(src).and_then(|pkg| emit_ir(&pkg, &config))
}

pub fn compile_string(src: &str) -> Result<String, String> {
    let config = Config::test();
    emit_string(src)
        .map_err(|err| err.to_string())
        .and_then(|ir| X86Builder::new(&config).assemble(ir))
        .and_then(|unit| Ok(unit.source))
}

pub fn debug_print_all_steps(src: &str) {
    let config = Config::default();
    let source = Source::new_from_string(src);

    scan(&source, &config)
        .and_then(|toks| parse(source, toks, &config))
        .and_then(|file| {
            Printer::print(&file);
            check(vec![file], &config)
        })
        .and_then(|pkg| emit_ir(&pkg, &config))
        .map_err(|err| err.to_string())
        .and_then(|unit| {
            print_ir(&unit.ins);
            X86Builder::new(&config).assemble(unit)
        })
        .map_or_else(
            |err| println!("{}", err),
            |unit| println!("{}", unit.source),
        );
}
