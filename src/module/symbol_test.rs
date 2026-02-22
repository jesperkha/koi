use crate::{
    ast::Pos,
    module::{FuncSymbol, ModulePath, Symbol, SymbolKind, SymbolOrigin},
};

#[test]
fn test_symbol_link_name_module() {
    let modpath = ModulePath::from("app.utils");
    let symbol = Symbol {
        filename: String::from("test/test.rs"),
        pos: Pos::default(),
        kind: SymbolKind::Function(FuncSymbol {
            is_inline: false,
            is_naked: false,
        }),
        ty: 0,
        name: String::from("helper"),
        origin: SymbolOrigin::Module(modpath.clone()),
        is_exported: true,
        no_mangle: false,
    };

    assert_eq!(symbol.link_name(), "_app_utils_helper");
}

#[test]
fn test_symbol_link_name_extern() {
    let modpath = ModulePath::from("app.utils");
    let extern_symbol = Symbol {
        filename: String::from("test/test.rs"),
        pos: Pos::default(),
        kind: SymbolKind::Function(FuncSymbol {
            is_inline: false,
            is_naked: false,
        }),
        ty: 0,
        name: String::from("external_func"),
        origin: SymbolOrigin::Extern(modpath),
        is_exported: false,
        no_mangle: false,
    };

    assert_eq!(extern_symbol.link_name(), "external_func");
}

#[test]
fn test_symbol_link_name_main() {
    let main_symbol = Symbol {
        filename: String::from("test/test.rs"),
        pos: Pos::default(),
        kind: SymbolKind::Function(FuncSymbol {
            is_inline: false,
            is_naked: false,
        }),
        ty: 0,
        name: String::from("main"),
        origin: SymbolOrigin::Module("app".into()),
        is_exported: true,
        no_mangle: false,
    };

    assert_eq!(main_symbol.link_name(), "main");
}
