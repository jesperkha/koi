use crate::ast::Printer;
use crate::util::{compare_string_lines_or_panic, must, parse_string};

fn compare_string(src: &str) {
    let ast = must(parse_string(src));
    let pstr = Printer::to_string(&ast);
    compare_string_lines_or_panic(pstr, src.to_string());
}

fn assert_pass(src: &str) {
    let _ = must(parse_string(src));
}

fn expect_error(src: &str, error: &str) {
    if let Err(e) = parse_string(src) {
        assert_eq!(e.len(), 1);
        assert_eq!(e.get(0).message, error);
    } else {
        panic!("expected error");
    }
}

#[test]
fn test_literal() {
    compare_string(
        r#"
        func f() {
            foo
            bar
            faz
        }
    "#,
    );
    compare_string(
        r#"
        func f() {
            (123)
            ((abc))
        }
    "#,
    );
}

#[test]
fn test_function_with_return() {
    compare_string(
        r#"
        func f() {
            return
        }
    "#,
    );
    compare_string(
        r#"
        func f() int {
            return 0
        }
    "#,
    );
    compare_string(
        r#"
        func f(a int) int {
            return 0
        }
    "#,
    );
    compare_string(
        r#"
        func f(a int, b bool, c float) int {
            return 0
        }
    "#,
    );
    compare_string(
        r#"
        func f() {
        }
    "#,
    );
    compare_string(
        r#"
        func f() bool {
            return false
        }
    "#,
    );
}

#[test]
fn test_function_with_error() {
    expect_error(
        r#"
        func f()
    "#,
        "expected {",
    );
    expect_error(
        r#"
        func f() {
    "#,
        "unexpected end of file while parsing block",
    );
    expect_error(
        r#"
        func f() {

        func g() {}
    "#,
        "expected expression",
    );
    expect_error(
        r#"
        func f( {}
    "#,
        "expected parameter name",
    );
    expect_error(
        r#"
        func f) {}
    "#,
        "expected (",
    );
    expect_error(
        r#"
        func f(foo) {}
    "#,
        "expected type",
    );
    expect_error(
        r#"
        func f(n int, n int) {}
    "#,
        "duplicate parameter name",
    );
}

#[test]
fn test_package_decl() {
    expect_error(
        r#"
        package
    "#,
        "expected package name",
    );
}

#[test]
fn test_function_call() {
    compare_string(
        r#"
        func f() {
            f()
        }
    "#,
    );
    compare_string(
        r#"
        func f() {
            f(1)
        }
    "#,
    );
    compare_string(
        r#"
        func f() {
            f(1, 2, true, abc)
        }
    "#,
    );
    compare_string(
        r#"
        func f() {
            a(b(d), b(c(d)))
        }
    "#,
    );
}

#[test]
fn test_complex_function_call() {
    compare_string(
        r#"
        func f() {
            f()()()
        }
    "#,
    );
    compare_string(
        r#"
        func f() {
            a(b()(c))(c, d())
        }
    "#,
    );
    compare_string(
        r#"
        func f() {
            ((a()(b()))()(a()))(a)
        }
    "#,
    );
}

#[test]
fn test_extern() {
    compare_string(
        r#"
        extern func write(fd int, s string, len int) int
    "#,
    );
}

#[test]
fn test_variable_decl() {
    compare_string(
        r#"
        func f() {
            a := 0
            b := true
            c :: 1.23
        }
    "#,
    );
    compare_string(
        r#"
        func f() {
            a := 0
            b := a
        }
    "#,
    );
    compare_string(
        r#"
        func f() int {
            a := 0
            return a
        }
    "#,
    );
}

#[test]
fn test_variable_decl_error() {
    expect_error(
        r#"
        func f() {
            a :=
        }
    "#,
        "expected expression",
    );
    expect_error(
        r#"
        func f() {
            a ::
        }
    "#,
        "expected expression",
    );
    expect_error(
        r#"
        func f() {
            1 := 1
        }
    "#,
        "invalid left hand value in declaration",
    );
    expect_error(
        r#"
        func f() {
            f() := 1
        }
    "#,
        "invalid left hand value in declaration",
    );
}

#[test]
fn test_variable_assign() {
    compare_string(
        r#"
        func f() {
            a = 0
            b = true
            c = b
        }
    "#,
    );
}

#[test]
fn test_imports() {
    compare_string(
        r#"
        import foo
    "#,
    );
    compare_string(
        r#"
        import foo.bar.faz
    "#,
    );
    compare_string(
        r#"
        import foo as bar
    "#,
    );
    compare_string(
        r#"
        import foo.bar as bar
    "#,
    );
    compare_string(
        r#"
        import foo {
            Foo,
            Bar
        }
    "#,
    );
    compare_string(
        r#"
        import foo.bar {
            Foo,
            Bar
        }
    "#,
    );
    assert_pass(
        r#"
        import foo.bar{
            Foo,
            Bar, }
    "#,
    );
    assert_pass(
        r#"
        import foo { Foo, Bar }
    "#,
    );
}
