use std::fmt::Display;

use crate::{
    ast::{Ast, Source, SourceMap, Token},
    config::Config,
    context::Context,
    error::Diagnostics,
    ir::Unit,
    lower::emit_ir,
    module::{ImportPath, ModuleId, ModulePath},
    parser::parse_source_map,
    scanner::scan,
    typecheck::check_fileset,
};

pub struct ErrorStream {
    pub errors: Vec<Error>,
}

impl ErrorStream {
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn get(&self, i: usize) -> &Error {
        &self.errors[i]
    }
}

pub struct Error {
    pub message: String,
}

impl From<Diagnostics> for ErrorStream {
    fn from(diag: Diagnostics) -> Self {
        let errors = diag
            .reports()
            .iter()
            .map(|report| Error {
                message: format!("{}", report.message),
            })
            .collect();
        ErrorStream { errors }
    }
}

impl Display for ErrorStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.errors
                .iter()
                .map(|err| err.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

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

pub fn must<T, V: Display>(res: Result<T, V>) -> T {
    res.unwrap_or_else(|err| panic!("unexpected error: {}", err))
}

pub fn scan_string(src: &str) -> Result<Vec<Token>, ErrorStream> {
    let map = new_source_map(src);
    let config = Config::test();
    scan(map.sources().last().unwrap(), &config).map_err(|e| e.into())
}

pub fn new_modpath(path: &str) -> ModulePath {
    let modpath: ModulePath = ImportPath::from(path).into();
    if path == "main" {
        modpath.to_main()
    } else {
        modpath
    }
}

pub fn parse_string(src: &str) -> Result<Ast, ErrorStream> {
    let map = new_source_map(src);
    let config = Config::test();
    parse_source_map(new_modpath("main"), &map, &config)
        .map(|mut fs| fs.files.pop().unwrap().ast)
        .map_err(|e| e.into())
}

pub fn check_string(ctx: &mut Context, src: &str) -> Result<ModuleId, ErrorStream> {
    let map = new_source_map(src);
    let fs = parse_source_map(new_modpath("main"), &map, &ctx.config)
        .map_err(|e| ErrorStream::from(e))?;
    check_fileset(ctx, fs)
        .map_err(|e| ErrorStream::from(e))
        .map(|create| ctx.modules.add(create))
}

pub fn emit_string(src: &str) -> Result<Unit, ErrorStream> {
    let config = Config::test();
    let mut ctx = Context::new(config);
    check_string(&mut ctx, src).and_then(|id| emit_ir(&ctx, id).map_err(|e| e.into()))
}
