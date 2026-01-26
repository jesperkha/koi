use crate::{
    ast::{File, FileSet},
    config::Config,
    error::{ErrorSet, Res},
    ir::{Unit, emit_ir},
    module::{Module, ModuleGraph, ModulePath},
    parser::parse,
    token::{Source, Token, scan},
    types::{TypeContext, type_check},
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

pub fn check_string<'a>(
    src: &str,
    mg: &'a mut ModuleGraph,
    ctx: &mut TypeContext,
) -> Res<&'a Module> {
    let config = Config::test();
    let fs = FileSet::new(ModulePath::new_str("main"), vec![parse_string(src)?]);
    type_check(fs, mg, ctx, &config)
}

pub fn emit_string(src: &str) -> Res<Unit> {
    let config = Config::test();
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    check_string(src, &mut mg, &mut ctx).and_then(|pkg| emit_ir(&pkg, &ctx, &config))
}
