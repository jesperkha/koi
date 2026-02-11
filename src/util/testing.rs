use crate::{
    ast::{Ast, File, FileSet},
    config::Config,
    error::{Diagnostics, Res},
    ir::Unit,
    lower::emit_ir,
    module::{Module, ModuleGraph, ModulePath},
    parser::parse,
    token::{Source, SourceMap, Token, scan},
    typecheck::{Checker, Importer},
    types::TypeContext,
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

pub fn new_source_map(src: &str) -> SourceMap {
    let mut map = SourceMap::new();
    map.add(Source::new_from_string(src));
    map
}

pub fn new_source_map_from_files(files: &[&str]) -> SourceMap {
    let mut map = SourceMap::new();
    for f in files {
        map.add(Source::new_from_string(f));
    }
    map
}

pub fn must<T>(map: &SourceMap, res: Result<T, Diagnostics>) -> T {
    res.unwrap_or_else(|err| panic!("unexpected error: {}", err.render(map)))
}

pub fn scan_string(src: &str) -> Res<Vec<Token>> {
    let src = Source::new_from_string(src);
    let config = Config::test();
    scan(&src, &config)
}

pub fn parse_string(src: &str) -> Res<Ast> {
    let src = Source::new_from_string(src);
    let config = Config::test();
    scan(&src, &config).and_then(|toks| parse(toks, &config))
}

pub fn check_string<'a>(
    src: &str,
    mg: &'a mut ModuleGraph,
    ctx: &mut TypeContext,
) -> Res<&'a Module> {
    let config = Config::test();
    let fs = FileSet::new(
        ModulePath::new_str("main"),
        vec![File::new(&Source::new_from_string(src), parse_string(src)?)],
    );
    let importer = Importer::new(mg);
    let checker = Checker::new(ctx, &importer, &config);
    let create_module = checker.check(fs)?;
    Ok(mg.add(create_module))
}

pub fn emit_string(src: &str) -> Res<Unit> {
    let config = Config::test();
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    check_string(src, &mut mg, &mut ctx).and_then(|pkg| emit_ir(&pkg, &ctx, &config))
}
