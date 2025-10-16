use crate::ast::Printer;
use crate::util::{compare_string_lines_or_panic, must, parse_string};

fn compare_string(src: &str) {
    let ast = must(parse_string(src));
    let pstr = Printer::to_string(&ast);
    compare_string_lines_or_panic(pstr, src.to_string());
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
}

#[test]
fn test_package_decl() {
    expect_error(
        r#"
        package
    "#,
        "expected package name",
    );
    // expect_error(
    //     r#"
    //     package a
    //     package b
    // "#,
    //     "package can only be declared once",
    // );
    // expect_error(
    //     r#"
    //     func f() {}
    //     package p
    // "#,
    //     "expected package declaration first",
    // );
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
