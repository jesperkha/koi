use crate::{parser::Parser, scanner::Scanner, token::File};

use super::*;

fn check(input: &str) -> CheckResult {
    let file = File::new_test_file(input);
    let toks = Scanner::scan(&file).unwrap();
    Parser::parse(&file, toks).and_then(|ast| Checker::check(&ast, &file))
}

fn assert_pass(src: &str) {
    check(src).expect("expected no errors");
}

fn assert_error(src: &str, msg: &str) {
    match check(src) {
        Ok(_) => panic!("expected error: '{}'", msg),
        Err(errs) => {
            assert!(errs.size() == 1, "expected one error, got {}", errs.size());
            assert_eq!(errs.get(0).message, msg);
        }
    }
}

#[test]
fn test_return_type() {
    assert_pass(
        r#"
        func foo() int {
            return 0
        }

        func bar() bool {
            return true
        }
    "#,
    );
    assert_pass(
        r#"
        func foo(a int, b bool) int {
            return a
        }

        func bar(a int, b bool) bool {
            return b
        }
    "#,
    );
    assert_error(
        r#"
        func foo() int {
            return true
        }
    "#,
        "incorrect return type: expected i64, got bool",
    );
    assert_error(
        r#"
        func foo(a int) bool {
            return a
        }
    "#,
        "incorrect return type: expected bool, got i64",
    );
}

#[test]
fn test_missing_return() {
    assert_pass(
        r#"
        func foo() {
        }
    "#,
    );
    assert_error(
        r#"
        func foo() int {
        }
    "#,
        "missing return in function foo",
    );
}

#[test]
fn test_undeclared_param() {
    assert_error(
        r#"
        func foo() bool {
            return a
        }
    "#,
        "not declared",
    );
    assert_error(
        r#"
        func foo(a int) {
        }

        func bar() int {
            return a
        }
    "#,
        "not declared",
    );
}
