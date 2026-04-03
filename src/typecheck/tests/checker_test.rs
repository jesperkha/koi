use crate::{
    config::Config,
    context::Context,
    util::{check_string, must},
};

fn assert_pass(src: &str) {
    let mut ctx = Context::new(Config::test());
    must(check_string(&mut ctx, src));
}

fn assert_error(src: &str, msg: &str) {
    let mut ctx = Context::new(Config::test());
    match check_string(&mut ctx, src) {
        Ok(_) => panic!("expected error: '{}'", msg),
        Err(errs) => {
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
        "incorrect return type: expected 'i32', got 'bool'",
    );
    assert_error(
        r#"
        func foo(a int) bool {
            return a
        }
    "#,
        "incorrect return type: expected 'bool', got 'i32'",
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
fn test_redefinition() {
    assert_error(
        r#"
        func f() {
        }
        func f() {
        }
    "#,
        "already declared",
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
        "mismatched types in function call. expected 'i32', got 'bool'",
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
        "incorrect return type: expected 'i32', got 'bool'",
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
        "mismatched types in assignment. expected 'i32', got 'bool'",
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
        "main function must return 'i32'",
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

#[test]
fn test_member_error() {
    assert_error(
        r#"
        func main() int {
            123.foo
            return 0
        }
    "#,
        "type 'i32' has no fields",
    );
    assert_error(
        r#"
        func f() string {
            return "foo"
        }
        func main() int {
            f().bar
            return 0
        }
    "#,
        "type 'string' has no fields",
    );
}

#[test]
fn test_duplicate_symbol() {
    assert_error(
        r#"
        func f() {}
        func f() {}
    "#,
        "already declared",
    );
}

#[test]
fn test_param_shadowing_is_error() {
    // declaring a local variable with the same name as a parameter should error
    assert_error(
        r#"
        func f(a int) {
            a := 1
        }
    "#,
        "already declared",
    );
}

#[test]
fn test_extern_then_func_conflict() {
    // extern declaration followed by a concrete function with same name should be an error
    assert_error(
        r#"
        extern func foo()
        func foo() {
        }
    "#,
        "already declared",
    );
}

#[test]
fn test_duplicate_extern_declaration() {
    // two extern declarations with same name should be an error
    assert_error(
        r#"
        extern func foo()
        extern func foo()
    "#,
        "already declared",
    );
}

#[test]
fn test_return_from_call_mismatch() {
    // returning result of a function with wrong return type should surface proper error
    assert_error(
        r#"
        func g() bool {
            return true
        }
        func f() int {
            return g()
        }
    "#,
        "incorrect return type: expected 'i32', got 'bool'",
    );
}

#[test]
fn test_binary_arithmetic_pass() {
    // Arithmetic ops preserve operand type
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a + b
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a - b
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a * b
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a / b
        }
    "#,
    );
    // Chained arithmetic
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a + b + a
        }
    "#,
    );
}

#[test]
fn test_binary_comparison_yields_bool() {
    // All comparison ops produce bool regardless of operand type
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a == b
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a != b
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a < b
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a > b
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a <= b
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a >= b
        }
    "#,
    );
}

#[test]
fn test_binary_comparison_not_int() {
    // Comparison result is bool, not int
    assert_error(
        r#"
        func f(a int, b int) int {
            return a == b
        }
    "#,
        "incorrect return type: expected 'i32', got 'bool'",
    );
    assert_error(
        r#"
        func f(a int, b int) int {
            return a < b
        }
    "#,
        "incorrect return type: expected 'i32', got 'bool'",
    );
}

#[test]
fn test_binary_logical_ops() {
    // && and || on bools produce bool
    assert_pass(
        r#"
        func f(a bool, b bool) bool {
            return a && b
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a bool, b bool) bool {
            return a || b
        }
    "#,
    );
    // Chained logical
    assert_pass(
        r#"
        func f(a bool, b bool) bool {
            return a && b || a
        }
    "#,
    );
}

#[test]
fn test_binary_modulo_yields_u32() {
    // Modulo produces u32
    assert_pass(
        r#"
        func f(a int, b int) {
            c := a % b
        }
    "#,
    );
    // Modulo result is u32, not int
    assert_error(
        r#"
        func f(a int, b int) int {
            return a % b
        }
    "#,
        "incorrect return type: expected 'i32', got 'u32'",
    );
}

#[test]
fn test_binary_type_mismatch_error() {
    assert_error(
        r#"
        func f(a int, b bool) int {
            return a + b
        }
    "#,
        "mismatched types in expression: 'i32' and 'bool'",
    );
    assert_error(
        r#"
        func f(a int, b bool) bool {
            return a == b
        }
    "#,
        "mismatched types in expression: 'i32' and 'bool'",
    );
    assert_error(
        r#"
        func f(a bool, b int) bool {
            return a && b
        }
    "#,
        "mismatched types in expression: 'bool' and 'i32'",
    );
}

#[test]
fn test_binary_result_as_variable() {
    // Assign comparison result (bool) to variable and use it
    assert_pass(
        r#"
        func f(a int, b int) bool {
            c := a == b
            return c
        }
    "#,
    );
    // Assign arithmetic result (int) to variable
    assert_pass(
        r#"
        func f(a int, b int) int {
            c := a + b
            return c
        }
    "#,
    );
}

#[test]
fn test_binary_result_as_function_argument() {
    // Pass bool result of comparison to function expecting bool
    assert_pass(
        r#"
        func consume(v bool) {}
        func f(a int, b int) {
            consume(a == b)
        }
    "#,
    );
    // Type mismatch: pass bool where int expected
    assert_error(
        r#"
        func consume(v int) {}
        func f(a int, b int) {
            consume(a == b)
        }
    "#,
        "mismatched types in function call. expected 'i32', got 'bool'",
    );
}

#[test]
fn test_binary_bool_equality() {
    // == on bools produces bool
    assert_pass(
        r#"
        func f(a bool, b bool) bool {
            return a == b
        }
    "#,
    );
}

#[test]
fn test_unary_not_pass() {
    assert_pass(
        r#"
        func f(a bool) bool {
            return !a
        }
    "#,
    );
    // Double negation
    assert_pass(
        r#"
        func f(a bool) bool {
            return !!a
        }
    "#,
    );
    // ! on comparison result
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return !(a == b)
        }
    "#,
    );
    // ! result assigned to variable
    assert_pass(
        r#"
        func f(a bool) bool {
            b := !a
            return b
        }
    "#,
    );
}

#[test]
fn test_unary_not_type_error() {
    assert_error(
        r#"
        func f(a int) bool {
            return !a
        }
    "#,
        "'!' operator can only be used on type 'bool', got 'i32'",
    );
    assert_error(
        r#"
        func f() bool {
            return !1
        }
    "#,
        "'!' operator can only be used on type 'bool', got 'i32'",
    );
}

#[test]
fn test_unary_not_yields_bool() {
    // ! result is bool, not int
    assert_error(
        r#"
        func f(a bool) int {
            return !a
        }
    "#,
        "incorrect return type: expected 'i32', got 'bool'",
    );
}

#[test]
fn test_unary_minus_pass() {
    assert_pass(
        r#"
        func f(a int) int {
            return -a
        }
    "#,
    );
    // Double negation
    assert_pass(
        r#"
        func f(a int) int {
            return --a
        }
    "#,
    );
    // - result assigned to variable
    assert_pass(
        r#"
        func f(a int) int {
            b := -a
            return b
        }
    "#,
    );
    // - in arithmetic expression
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a + -b
        }
    "#,
    );
}

#[test]
fn test_unary_minus_preserves_type() {
    // - preserves the operand type, not bool
    assert_error(
        r#"
        func f(a int) bool {
            return -a
        }
    "#,
        "incorrect return type: expected 'bool', got 'i32'",
    );
}

#[test]
fn test_unary_minus_type_error() {
    assert_error(
        r#"
        func f(a bool) bool {
            return -a
        }
    "#,
        "'-' operator can only be used on number types, got 'bool'",
    );
    assert_error(
        r#"
        func f(s string) string {
            return -s
        }
    "#,
        "'-' operator can only be used on number types, got 'string'",
    );
}

#[test]
fn test_if_stmt_pass() {
    assert_pass(
        r#"
        func f(a bool) {
            if a {
            }
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) {
            if a == b {
            }
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a bool) {
            if a {
            } else {
            }
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a bool, b bool) {
            if a {
            } else if b {
            } else {
            }
        }
    "#,
    );
    assert_pass(
        r#"
        func f(a int, b int) {
            if a < b {
                a = 0
            }
        }
    "#,
    );
}

#[test]
fn test_if_stmt_condition_must_be_bool() {
    assert_error(
        r#"
        func f(a int) {
            if a {
            }
        }
    "#,
        "expression must be of type 'bool', got 'i32'",
    );
    assert_error(
        r#"
        func f() {
            if 1 {
            }
        }
    "#,
        "expression must be of type 'bool', got 'i32'",
    );
    assert_error(
        r#"
        func f(s string) {
            if s {
            }
        }
    "#,
        "expression must be of type 'bool', got 'string'",
    );
    assert_error(
        r#"
        func g() int { return 0 }
        func f() {
            if g() {
            }
        }
    "#,
        "expression must be of type 'bool', got 'i32'",
    );
}

#[test]
fn test_if_elseif_condition_must_be_bool() {
    assert_error(
        r#"
        func f(a bool, b int) {
            if a {
            } else if b {
            }
        }
    "#,
        "expression must be of type 'bool', got 'i32'",
    );
}

#[test]
fn test_if_block_scope() {
    // variable declared inside if block is not visible outside
    assert_error(
        r#"
        func f(a bool) int {
            if a {
                x := 1
            }
            return x
        }
    "#,
        "not declared",
    );
    // variable declared in outer scope is accessible inside if block
    assert_pass(
        r#"
        func f(a bool) int {
            x := 0
            if a {
                x = 1
            }
            return x
        }
    "#,
    );
}

#[test]
fn test_if_stmt_body_is_checked() {
    // type errors inside the if body are still caught
    assert_error(
        r#"
        func f(a bool, b int) {
            if a {
                b = true
            }
        }
    "#,
        "mismatched types in assignment. expected 'i32', got 'bool'",
    );
}

#[test]
fn test_unary_as_function_argument() {
    assert_pass(
        r#"
        func consume(v bool) {}
        func f(a bool) {
            consume(!a)
        }
    "#,
    );
    assert_pass(
        r#"
        func consume(v int) {}
        func f(a int) {
            consume(-a)
        }
    "#,
    );
    // Type mismatch: ! yields bool but int expected
    assert_error(
        r#"
        func consume(v int) {}
        func f(a bool) {
            consume(!a)
        }
    "#,
        "mismatched types in function call. expected 'i32', got 'bool'",
    );
}

#[test]
fn test_return_only_in_if_branch_is_not_exhaustive() {
    // A return inside an if-only block does not guarantee a return —
    // the else path is missing, so this should be a "missing return" error.
    assert_error(
        r#"
        func f(a bool) int {
            if a {
                return 1
            }
        }
    "#,
        "missing return in function 'f'",
    );
}

// --- Exhaustive return checks through if-else chains ---

#[test]
fn test_if_else_both_return_is_exhaustive() {
    // Both branches return → should pass
    assert_pass(
        r#"
        func f(a bool) int {
            if a {
                return 1
            } else {
                return 0
            }
        }
    "#,
    );
}

#[test]
fn test_if_elseif_else_all_return_is_exhaustive() {
    // All three branches return → should pass
    assert_pass(
        r#"
        func f(a bool, b bool) int {
            if a {
                return 1
            } else if b {
                return 2
            } else {
                return 3
            }
        }
    "#,
    );
}

#[test]
fn test_if_else_missing_return_in_if_branch() {
    // else returns but if branch does not → not exhaustive
    assert_error(
        r#"
        func f(a bool) int {
            if a {
            } else {
                return 0
            }
        }
    "#,
        "missing return in function 'f'",
    );
}

#[test]
fn test_if_elseif_no_terminal_else_not_exhaustive() {
    // No terminal else: if the else-if condition is also false nothing returns
    assert_error(
        r#"
        func f(a bool, b bool) int {
            if a {
                return 1
            } else if b {
                return 2
            }
        }
    "#,
        "missing return in function 'f'",
    );
}

#[test]
fn test_if_elseif_missing_return_in_elseif_branch() {
    // else-if branch has no return → not exhaustive
    assert_error(
        r#"
        func f(a bool, b bool) int {
            if a {
                return 1
            } else if b {
            } else {
                return 3
            }
        }
    "#,
        "missing return in function 'f'",
    );
}

#[test]
fn test_if_elseif_missing_return_in_else_branch() {
    // else branch has no return → not exhaustive
    assert_error(
        r#"
        func f(a bool, b bool) int {
            if a {
                return 1
            } else if b {
                return 2
            } else {
            }
        }
    "#,
        "missing return in function 'f'",
    );
}

// --- Nested if-else return exhaustiveness ---

#[test]
fn test_nested_if_else_exhaustive() {
    // Inner if-else covers both paths of the outer if branch → exhaustive
    assert_pass(
        r#"
        func f(a bool, b bool) int {
            if a {
                if b {
                    return 1
                } else {
                    return 2
                }
            } else {
                return 3
            }
        }
    "#,
    );
    // Both branches of outer if-else contain exhaustive nested if-else
    assert_pass(
        r#"
        func f(a bool, b bool) int {
            if a {
                if b {
                    return 1
                } else {
                    return 2
                }
            } else {
                if b {
                    return 3
                } else {
                    return 4
                }
            }
        }
    "#,
    );
}

#[test]
fn test_nested_if_only_inside_branch_not_exhaustive() {
    // Inner if has no else: the b=false path through the outer if branch doesn't return
    assert_error(
        r#"
        func f(a bool, b bool) int {
            if a {
                if b {
                    return 1
                }
            } else {
                return 3
            }
        }
    "#,
        "missing return in function 'f'",
    );
}

#[test]
fn test_nested_exhaustive_if_else_without_outer_else_not_exhaustive() {
    // Inner if-else is exhaustive, but the outer has no else → the a=false path doesn't return
    assert_error(
        r#"
        func f(a bool, b bool) int {
            if a {
                if b {
                    return 1
                } else {
                    return 2
                }
            }
        }
    "#,
        "missing return in function 'f'",
    );
}

#[test]
fn test_nested_if_else_one_inner_branch_missing_return() {
    // Outer else branch has a nested if-else where one arm doesn't return
    assert_error(
        r#"
        func f(a bool, b bool) int {
            if a {
                return 1
            } else {
                if b {
                    return 2
                } else {
                }
            }
        }
    "#,
        "missing return in function 'f'",
    );
}
