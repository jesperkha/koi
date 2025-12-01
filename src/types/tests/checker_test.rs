use crate::{
    types::Package,
    util::{check_string, must},
};

fn assert_pass(src: &str) -> Package {
    must(check_string(src))
}

fn assert_error(src: &str, msg: &str) {
    match check_string(src) {
        Ok(_) => panic!("expected error: '{}'", msg),
        Err(errs) => {
            assert!(errs.len() == 1, "expected one error, got {}", errs.len());
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
        "incorrect return type: expected 'i64', got 'bool'",
    );
    assert_error(
        r#"
        func foo(a int) bool {
            return a
        }
    "#,
        "incorrect return type: expected 'bool', got 'i64'",
    );
    assert_error(
        r#"
        func foo() bar {
        }
    "#,
        "not a type",
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
        "missing return in function 'foo'",
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

#[test]
fn test_function_call_pass() {
    assert_pass(
        r#"
        func f(a int, b int) int {
            return 0
        }
        func g() int {
            return f(1, 2)
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int) int {
            return f(f(f(1)))
        }
    "#,
    );
}

#[test]
fn test_function_call_fail() {
    assert_error(
        r#"
        func f() {
            g()
        }
    "#,
        "not declared",
    );
    assert_error(
        r#"
        func f() {
            (1)()
        }
    "#,
        "not a function",
    );
    assert_error(
        r#"
        func f() {
            f()()
        }
    "#,
        "not a function",
    );
    assert_error(
        r#"
        func f(a int, b int) {
            f(1)
        }
    "#,
        "function takes 2 arguments, got 1",
    );
    assert_error(
        r#"
        func f(a int) {
            f(true)
        }
    "#,
        "mismatched types in function call. expected 'i64', got 'bool'",
    );
}

#[test]
fn test_string_literal() {
    assert_pass(
        r#"
        func f(s string) string {
            return s
        }
    "#,
    );
}

#[test]
fn test_extern() {
    assert_pass(
        r#"
        extern func foo()

        func f() {
            foo()
        }
    "#,
    );
    assert_pass(
        r#"
        extern func write(fd int, s string, len int) int

        func f() {
            write(1, "Hello", 5) 
        }
    "#,
    );
}

#[test]
fn test_variable_decl() {
    assert_pass(
        r#"
        func f() {
            a := 0
        }
    "#,
    );
    assert_pass(
        r#"
        func f() {
            a := 0
            b :: a
        }
    "#,
    );
    assert_pass(
        r#"
        func f() bool {
            a := true
            return a
        }
    "#,
    );
    assert_pass(
        r#"
        func f(n int) int {
            a := f(n)
            f(a)
            return f(a)
        }
    "#,
    );
}

#[test]
fn test_variable_decl_error() {
    assert_error(
        r#"
        func f() int {
            a := true
            return a
        }
    "#,
        "incorrect return type: expected 'i64', got 'bool'",
    );
    assert_error(
        r#"
        func f() {
            a := true
            a := true
        }
    "#,
        "already declared",
    );
    assert_error(
        r#"
        func g() {}
        func f() {
            a := g()
        }
    "#,
        "cannot assign void type to variable",
    );
}

#[test]
fn test_variable_assignment() {
    assert_pass(
        r#"
        func f() {
            a := 0
            a = 1
            a = 2
        }
    "#,
    );
    assert_pass(
        r#"
        func f() int {
            a := 0
            a = f()
            return a
        }
    "#,
    );
    assert_pass(
        r#"
        func f() {
            a :: 0
            b := a
            b = 1
        }
    "#,
    );
}

#[test]
fn test_variable_assignment_fail() {
    assert_error(
        r#"
        func f() {
            a := 0
            a = true
        }
    "#,
        "mismatched types in assignment. expected 'i64', got 'bool'",
    );
    assert_error(
        r#"
        func f() {
            a = 1
        }
    "#,
        "not declared",
    );
    assert_error(
        r#"
        func f() {
            a :: 0
            a = 1
        }
    "#,
        "cannot assign new value to a constant",
    );
}

#[test]
fn test_main_function_rules() {
    assert_error(
        r#"
        func main() {
            return
        }
    "#,
        "main function must return 'i64'",
    );
    assert_error(
        r#"
        func main(a int) int {
            return 0
        }
    "#,
        "main function must not take any arguments",
    );
}
