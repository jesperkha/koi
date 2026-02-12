use crate::{
    ast::{Ast, File, FileSet, Source, SourceMap, Token},
    config::Config,
    error::{Diagnostics, Res},
    ir::Unit,
    lower::emit_ir,
    module::{Module, ModuleGraph, ModulePath},
    parser::parse_source_map,
    scanner::scan,
    typecheck::check_fileset,
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

pub fn new_source(src: &str) -> Source {
    Source::new_str("test".into(), src.into())
}

pub fn new_source_map(src: &str) -> SourceMap {
    let mut map = SourceMap::new();
    map.add(new_source(src));
    map
}

pub fn new_source_map_from_files(files: &[&str]) -> SourceMap {
    let mut map = SourceMap::new();
    for f in files {
        map.add(new_source(f));
    }
    map
}

pub fn must<T>(map: &SourceMap, res: Result<T, Diagnostics>) -> T {
    res.unwrap_or_else(|err| panic!("unexpected error: {}", err.render(map)))
}

pub fn scan_string(src: &str) -> Res<Vec<Token>> {
    let src = new_source(src);
    let config = Config::test();
    scan(&src, &config)
}

pub fn parse_string(src: &str) -> Res<Ast> {
    let map = new_source_map(src);
    let config = Config::test();
    parse_source_map(ModulePath::new("test".into()), &map, &config)
        .map(|mut fs| fs.files.pop().unwrap().ast)
}

pub fn check_string<'a>(
    src: &str,
    mg: &'a mut ModuleGraph,
    ctx: &mut TypeContext,
) -> Res<&'a Module> {
    let config = Config::test();
    let fs = FileSet::new(
        ModulePath::new_str("main"),
        vec![File::new(&new_source(src), parse_string(src)?)],
    );
    let create_module = check_fileset(fs, mg, ctx, &config)?;
    Ok(mg.add(create_module))
}

pub fn emit_string(src: &str) -> Res<Unit> {
    let config = Config::test();
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    check_string(src, &mut mg, &mut ctx).and_then(|pkg| emit_ir(&pkg, &ctx, &config))
}
