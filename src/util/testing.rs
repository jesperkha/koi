use crate::{
    ast::File,
    config::Config,
    error::{ErrorSet, Res},
    ir::{IRUnit, emit_ir},
    parser::parse,
    pkg::Package,
    scanner::scan,
    token::{Source, Token},
    types::check,
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
    scan(&src)
}

pub fn parse_string(src: &str) -> Res<File> {
    let src = Source::new_from_string(src);
    let config = Config::test();
    scan(&src).and_then(|toks| parse(src, toks, &config))
}

pub fn check_string(src: &str) -> Res<Package> {
    check(vec![parse_string(src)?])
}

pub fn emit_string(src: &str) -> Res<IRUnit> {
    check_string(src).and_then(|pkg| emit_ir(&pkg))
}
