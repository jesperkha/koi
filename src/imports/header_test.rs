use std::vec;

use crate::{
    imports::{create_header_file, read_header_file},
    module::{CreateModule, ModuleGraph, ModulePath},
    types::{FunctionType, PrimitiveType, TypeContext, TypeId, TypeKind},
    util::{check_string, must, new_modpath},
};

fn create_header_module(
    src: &str,
    modpath: ModulePath,
    mg: &mut ModuleGraph,
    ctx: &mut TypeContext,
) -> CreateModule {
    let module = must(check_string(src, mg, ctx));
    let header = must(create_header_file(module, &ctx));
    must(read_header_file(modpath, &header, ctx))
}

fn func_type_id(ctx: &mut TypeContext, params: &[PrimitiveType], ret: PrimitiveType) -> TypeId {
    ctx.get_or_intern(TypeKind::Function(FunctionType {
        params: params.iter().map(|p| ctx.primitive(p.clone())).collect(),
        ret: ctx.primitive(ret),
    }))
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
    let create_mod = create_header_module(src, new_modpath("lib.test"), &mut mg, &mut ctx);

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

#[test]
fn test_loading_header_module() {
    let src = r#"
    pub func doFoo() int {
        return 0
    }
    "#;
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    let create_mod = create_header_module(src, new_modpath("foo"), &mut mg, &mut ctx);

    let foo = must(create_mod.symbols.get("doFoo"));
    assert_eq!(foo.ty, func_type_id(&mut ctx, &vec![], PrimitiveType::I64));

    mg.add(create_mod);

    let src2 = r#"
    import foo

    func main() int {
        foo.doFoo()
        return 0
    }
    "#;

    must(check_string(src2, &mut mg, &mut ctx));
}

#[test]
fn test_multiple_modules() {
    let src1 = r#"
    pub func doFoo() string {
        return ""
    }
    "#;
    let src2 = r#"
    pub func doBar() string {
        return ""
    }
    "#;

    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();
    let create_mod1 = create_header_module(src1, new_modpath("foo"), &mut mg, &mut ctx);
    let create_mod2 = create_header_module(src2, new_modpath("bar"), &mut mg, &mut ctx);

    let do_foo = must(create_mod1.symbols.get("doFoo")).ty;
    let do_bar = must(create_mod2.symbols.get("doBar")).ty;

    mg.add(create_mod1);
    mg.add(create_mod2);

    let src3 = r#"
    import foo
    import bar

    func main() int {
        foo.doFoo()
        bar.doBar()
        return 0
    }

    func doFaz() string {
        return ""
    }
    "#;

    let module3 = must(check_string(src3, &mut mg, &mut ctx));
    let do_faz = must(module3.symbols.get("doFaz")).ty;

    assert_eq!(do_foo, do_bar);
    assert_eq!(do_bar, do_faz);
    assert_eq!(
        do_faz,
        func_type_id(&mut ctx, &vec![], PrimitiveType::String)
    );
}
