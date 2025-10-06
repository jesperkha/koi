use crate::ast::{Ast, Printer};
use crate::error::ErrorSet;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::token::File;
use crate::util::{compare_string_lines_or_panic, must};

fn parse(src: &str) -> Result<Ast, ErrorSet> {
    let file = File::new_test_file(src);
    let toks = must(Scanner::scan(&file));
    Parser::parse(&file, toks)
}

fn compare_string(src: &str) {
    let ast = must(parse(src));
    let pstr = Printer::to_string(&ast);
    compare_string_lines_or_panic(pstr, src.to_string());
}

fn expect_error(src: &str, error: &str) {
    if let Err(e) = parse(src) {
        assert_eq!(e.size(), 1);
        assert_eq!(e.get(0).message, error);
    } else {
        panic!("expected error");
    }
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
    compare_string(
        r#"
        package p
    "#,
    );
    compare_string(
        r#"
        package p

        func f() {
        }
    "#,
    );
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
