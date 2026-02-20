use std::vec;

use crate::{
    imports::{create_header_file, read_header_file},
    module::ModuleGraph,
    types::{FunctionType, PrimitiveType, TypeContext, TypeId, TypeKind},
    util::{check_string, must},
};

#[test]
fn test_create_and_read_header() {
    let src = r#"
    pub func foo() int {
        return 0
    }

    pub func bar(a string, b bool) int {
        return 0
    }

    pub func faz(a string, b bool) int {
        return 0
    }
    "#;
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    let module = must(check_string(src, &mut mg, &mut ctx));
    let header = must(create_header_file(module, &ctx));
    let _ = must(read_header_file(&header, &mut ctx));
}

#[test]
fn test_read_header_file() {
    let src = r#"
    pub func foo() int {
        return 0
    }

    pub func bar(a string, b bool) int {
        return 0
    }

    pub func faz() {}
    "#;
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    let module = must(check_string(src, &mut mg, &mut ctx));

    let header = must(create_header_file(module, &ctx));
    let create_mod = must(read_header_file(&header, &mut ctx));

    let foo = must(create_mod.symbols.get("foo"));
    assert_eq!(foo.ty, func_type_id(&mut ctx, &vec![], PrimitiveType::I64));

    let bar = must(create_mod.symbols.get("bar"));
    assert_eq!(
        bar.ty,
        func_type_id(
            &mut ctx,
            &vec![PrimitiveType::String, PrimitiveType::Bool],
            PrimitiveType::I64
        )
    );

    let faz = must(create_mod.symbols.get("faz"));
    assert_eq!(faz.ty, func_type_id(&mut ctx, &vec![], PrimitiveType::Void));
}

fn func_type_id(ctx: &mut TypeContext, params: &[PrimitiveType], ret: PrimitiveType) -> TypeId {
    ctx.get_or_intern(TypeKind::Function(FunctionType {
        params: params.iter().map(|p| ctx.primitive(p.clone())).collect(),
        ret: ctx.primitive(ret),
    }))
}
