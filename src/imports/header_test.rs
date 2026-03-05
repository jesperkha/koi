use std::vec;

use crate::{
    config::Config,
    context::{Context, CreateModule},
    imports::{create_header_file, read_header_file},
    module::ModulePath,
    types::{FunctionType, PrimitiveType, TypeId, TypeKind},
    util::{check_string, must, new_modpath},
};

fn create_header_module<'a>(ctx: &'a mut Context, src: &str, modpath: ModulePath) -> CreateModule {
    let id = must(check_string(ctx, src));
    let header = must(create_header_file(ctx, id));
    must(read_header_file(ctx, modpath, &header))
}

fn func_type_id(ctx: &mut Context, params: &[PrimitiveType], ret: PrimitiveType) -> TypeId {
    ctx.types.get_or_intern(TypeKind::Function(FunctionType {
        params: params
            .iter()
            .map(|p| ctx.types.primitive(p.clone()))
            .collect(),
        ret: ctx.types.primitive(ret),
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
    let mut ctx = Context::new(Config::default());
    let create_mod = create_header_module(&mut ctx, src, new_modpath("lib.test"));

    let foo = ctx
        .symbols
        .get(create_mod.symbols.get("foo").unwrap().id)
        .ty;
    assert_eq!(foo, func_type_id(&mut ctx, &vec![], PrimitiveType::I64));

    let bar = ctx
        .symbols
        .get(create_mod.symbols.get("bar").unwrap().id)
        .ty;
    assert_eq!(
        bar,
        func_type_id(
            &mut ctx,
            &vec![PrimitiveType::String, PrimitiveType::Bool],
            PrimitiveType::I64
        )
    );

    let faz = ctx
        .symbols
        .get(create_mod.symbols.get("faz").unwrap().id)
        .ty;
    assert_eq!(faz, func_type_id(&mut ctx, &vec![], PrimitiveType::Void));
}

#[test]
fn test_loading_header_module() {
    let src = r#"
    pub func doFoo() int {
        return 0
    }
    "#;
    let mut ctx = Context::new(Config::test());
    let create_mod = create_header_module(&mut ctx, src, new_modpath("foo"));

    let foo = ctx
        .symbols
        .get(create_mod.symbols.get("doFoo").unwrap().id)
        .ty;
    assert_eq!(foo, func_type_id(&mut ctx, &vec![], PrimitiveType::I64));

    ctx.modules.add(create_mod);

    let src2 = r#"
    import foo

    func main() int {
        foo.doFoo()
        return 0
    }
    "#;

    must(check_string(&mut ctx, src2));
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

    let mut ctx = Context::new(Config::test());
    let create_mod1 = create_header_module(&mut ctx, src1, new_modpath("foo"));
    let create_mod2 = create_header_module(&mut ctx, src2, new_modpath("bar"));

    let do_foo = ctx
        .symbols
        .get(create_mod1.symbols.get("doFoo").unwrap().id)
        .ty;
    let do_bar = ctx
        .symbols
        .get(create_mod2.symbols.get("doBar").unwrap().id)
        .ty;

    ctx.modules.add(create_mod1);
    ctx.modules.add(create_mod2);

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

    let mod3id = must(check_string(&mut ctx, src3));
    let module3 = ctx.modules.get(mod3id);
    let do_faz = ctx.symbols.get(module3.symbols.get("doFaz").unwrap().id).ty;

    assert_eq!(do_foo, do_bar);
    assert_eq!(do_bar, do_faz);
    assert_eq!(
        do_faz,
        func_type_id(&mut ctx, &vec![], PrimitiveType::String)
    );
}
