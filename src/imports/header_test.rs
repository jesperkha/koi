use crate::{
    imports::{create_header_file, read_header_file},
    module::{ModuleGraph, ModuleKind},
    types::TypeContext,
    util::{check_string, must},
};

#[test]
fn test_create_and_read_header() {
    let src = r#""#;
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    let module = must(check_string(src, &mut mg, &mut ctx));

    let header = must(create_header_file(module, &ctx));
    let create_mod = must(read_header_file(&header, &mut mg, &mut ctx));

    assert!(matches!(create_mod.kind, ModuleKind::External(_)));
}

#[test]
fn test_read_header_file() {
    let src = r#"
    pub func foo() int {
        return 0
    }

    pub func bar(a string, b bool) int {
        return 0}

    pub func faz() {}
    "#;
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    let module = must(check_string(src, &mut mg, &mut ctx));

    let header = must(create_header_file(module, &ctx));
    let create_mod = must(read_header_file(&header, &mut mg, &mut ctx));

    // TODO: assert symbol info

    let foo = must(create_mod.symbols.get("foo"));
    assert_eq!(foo.is_exported, true);

    let bar = must(create_mod.symbols.get("bar"));
    assert_eq!(bar.is_exported, true);

    let faz = must(create_mod.symbols.get("faz"));
    assert_eq!(faz.is_exported, true);
}
