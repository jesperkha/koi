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
fn test_return_type_pass_basic() {
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
}

#[test]
fn test_return_type_pass_with_params() {
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
}

#[test]
fn test_return_type_error_bool_in_int_func() {
    assert_error(
        r#"
        func foo() int {
            return true
        }
    "#,
        "incorrect return type: expected 'i32', got 'bool'",
    );
}

#[test]
fn test_return_type_error_int_in_bool_func() {
    assert_error(
        r#"
        func foo(a int) bool {
            return a
        }
    "#,
        "incorrect return type: expected 'bool', got 'i32'",
    );
}

#[test]
fn test_return_type_error_unknown_type() {
    assert_error(
        r#"
        func foo() bar {
        }
    "#,
        "not a type",
    );
}

#[test]
fn test_missing_return_pass_void() {
    assert_pass(
        r#"
        func foo() {
        }
    "#,
    );
}

#[test]
fn test_missing_return_error() {
    assert_error(
        r#"
        func foo() int {
        }
    "#,
        "missing return in function 'foo'",
    );
}

#[test]
fn test_undeclared_variable_error() {
    assert_error(
        r#"
        func foo() bool {
            return a
        }
    "#,
        "not declared",
    );
}

#[test]
fn test_undeclared_variable_not_leaked_from_other_function() {
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
fn test_function_call_pass_with_args() {
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
}

#[test]
fn test_function_call_pass_recursive() {
    assert_pass(
        r#"
        func f(a int) int {
            return f(f(f(1)))
        }
    "#,
    );
}

#[test]
fn test_function_call_fail_not_declared() {
    assert_error(
        r#"
        func f() {
            g()
        }
    "#,
        "not declared",
    );
}

#[test]
fn test_function_call_fail_call_literal() {
    assert_error(
        r#"
        func f() {
            (1)()
        }
    "#,
        "not a function",
    );
}

#[test]
fn test_function_call_fail_call_void_result() {
    assert_error(
        r#"
        func f() {
            f()()
        }
    "#,
        "not a function",
    );
}

#[test]
fn test_function_call_fail_too_few_args() {
    assert_error(
        r#"
        func f(a int, b int) {
            f(1)
        }
    "#,
        "function takes 2 arguments, got 1",
    );
}

#[test]
fn test_function_call_fail_type_mismatch() {
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
fn test_extern_no_args() {
    assert_pass(
        r#"
        extern func foo()

        func f() {
            foo()
        }
    "#,
    );
}

#[test]
fn test_extern_with_args() {
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
fn test_variable_decl_pass_basic() {
    assert_pass(
        r#"
        func f() {
            a := 0
        }
    "#,
    );
}

#[test]
fn test_variable_decl_pass_copy() {
    assert_pass(
        r#"
        func f() {
            a := 0
            b :: a
        }
    "#,
    );
}

#[test]
fn test_variable_decl_pass_bool() {
    assert_pass(
        r#"
        func f() bool {
            a := true
            return a
        }
    "#,
    );
}

#[test]
fn test_variable_decl_pass_with_calls() {
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
fn test_variable_decl_error_wrong_return_type() {
    assert_error(
        r#"
        func f() int {
            a := true
            return a
        }
    "#,
        "incorrect return type: expected 'i32', got 'bool'",
    );
}

#[test]
fn test_variable_decl_error_redeclared() {
    assert_error(
        r#"
        func f() {
            a := true
            a := true
        }
    "#,
        "already declared",
    );
}

#[test]
fn test_variable_decl_error_assign_void() {
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
fn test_variable_assignment_pass_multiple() {
    assert_pass(
        r#"
        func f() {
            a := 0
            a = 1
            a = 2
        }
    "#,
    );
}

#[test]
fn test_variable_assignment_pass_from_call() {
    assert_pass(
        r#"
        func f() int {
            a := 0
            a = f()
            return a
        }
    "#,
    );
}

#[test]
fn test_variable_assignment_pass_const_then_mut() {
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
fn test_variable_assignment_fail_type_mismatch() {
    assert_error(
        r#"
        func f() {
            a := 0
            a = true
        }
    "#,
        "mismatched types in assignment. expected 'i32', got 'bool'",
    );
}

#[test]
fn test_variable_assignment_fail_not_declared() {
    assert_error(
        r#"
        func f() {
            a = 1
        }
    "#,
        "not declared",
    );
}

#[test]
fn test_variable_assignment_fail_reassign_const() {
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
fn test_main_function_must_return_i32() {
    assert_error(
        r#"
        func main() {
            return
        }
    "#,
        "main function must return 'i32', got 'void'",
    );
}

#[test]
fn test_main_function_must_not_take_args() {
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
fn test_member_error_on_int_literal() {
    assert_error(
        r#"
        func main() int {
            123.foo
            return 0
        }
    "#,
        "type 'i32' has no fields",
    );
}

#[test]
fn test_member_error_on_string() {
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
fn test_binary_add_pass() {
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a + b
        }
    "#,
    );
}

#[test]
fn test_binary_sub_pass() {
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a - b
        }
    "#,
    );
}

#[test]
fn test_binary_mul_pass() {
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a * b
        }
    "#,
    );
}

#[test]
fn test_binary_div_pass() {
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a / b
        }
    "#,
    );
}

#[test]
fn test_binary_add_chained_pass() {
    assert_pass(
        r#"
        func f(a int, b int) int {
            return a + b + a
        }
    "#,
    );
}

#[test]
fn test_binary_eq_yields_bool() {
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a == b
        }
    "#,
    );
}

#[test]
fn test_binary_ne_yields_bool() {
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a != b
        }
    "#,
    );
}

#[test]
fn test_binary_lt_yields_bool() {
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a < b
        }
    "#,
    );
}

#[test]
fn test_binary_gt_yields_bool() {
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a > b
        }
    "#,
    );
}

#[test]
fn test_binary_le_yields_bool() {
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a <= b
        }
    "#,
    );
}

#[test]
fn test_binary_ge_yields_bool() {
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return a >= b
        }
    "#,
    );
}

#[test]
fn test_binary_eq_not_int() {
    assert_error(
        r#"
        func f(a int, b int) int {
            return a == b
        }
    "#,
        "incorrect return type: expected 'i32', got 'bool'",
    );
}

#[test]
fn test_binary_lt_not_int() {
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
fn test_binary_and_pass() {
    assert_pass(
        r#"
        func f(a bool, b bool) bool {
            return a && b
        }
    "#,
    );
}

#[test]
fn test_binary_or_pass() {
    assert_pass(
        r#"
        func f(a bool, b bool) bool {
            return a || b
        }
    "#,
    );
}

#[test]
fn test_binary_and_or_chained_pass() {
    assert_pass(
        r#"
        func f(a bool, b bool) bool {
            return a && b || a
        }
    "#,
    );
}

#[test]
fn test_binary_modulo_pass() {
    assert_pass(
        r#"
        func f(a int, b int) {
            c := a % b
        }
    "#,
    );
}

#[test]
fn test_binary_modulo_error_used_as_int() {
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
fn test_binary_type_mismatch_arithmetic() {
    assert_error(
        r#"
        func f(a int, b bool) int {
            return a + b
        }
    "#,
        "mismatched types in expression: 'i32' and 'bool'",
    );
}

#[test]
fn test_binary_type_mismatch_comparison() {
    assert_error(
        r#"
        func f(a int, b bool) bool {
            return a == b
        }
    "#,
        "mismatched types in expression: 'i32' and 'bool'",
    );
}

#[test]
fn test_binary_type_mismatch_logical() {
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
fn test_binary_comparison_result_in_variable() {
    assert_pass(
        r#"
        func f(a int, b int) bool {
            c := a == b
            return c
        }
    "#,
    );
}

#[test]
fn test_binary_arithmetic_result_in_variable() {
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
fn test_binary_comparison_result_as_arg_pass() {
    assert_pass(
        r#"
        func consume(v bool) {}
        func f(a int, b int) {
            consume(a == b)
        }
    "#,
    );
}

#[test]
fn test_binary_comparison_result_as_arg_fail() {
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
fn test_unary_not_basic_pass() {
    assert_pass(
        r#"
        func f(a bool) bool {
            return !a
        }
    "#,
    );
}

#[test]
fn test_unary_not_double_pass() {
    assert_pass(
        r#"
        func f(a bool) bool {
            return !!a
        }
    "#,
    );
}

#[test]
fn test_unary_not_on_comparison_pass() {
    assert_pass(
        r#"
        func f(a int, b int) bool {
            return !(a == b)
        }
    "#,
    );
}

#[test]
fn test_unary_not_result_in_variable_pass() {
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
fn test_unary_not_error_on_int() {
    assert_error(
        r#"
        func f(a int) bool {
            return !a
        }
    "#,
        "'!' operator can only be used on type 'bool', got 'i32'",
    );
}

#[test]
fn test_unary_not_error_on_int_literal() {
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
fn test_unary_minus_basic_pass() {
    assert_pass(
        r#"
        func f(a int) int {
            return -a
        }
    "#,
    );
}

#[test]
fn test_unary_minus_double_pass() {
    assert_pass(
        r#"
        func f(a int) int {
            return --a
        }
    "#,
    );
}

#[test]
fn test_unary_minus_result_in_variable_pass() {
    assert_pass(
        r#"
        func f(a int) int {
            b := -a
            return b
        }
    "#,
    );
}

#[test]
fn test_unary_minus_in_expression_pass() {
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
fn test_unary_minus_error_on_bool() {
    assert_error(
        r#"
        func f(a bool) bool {
            return -a
        }
    "#,
        "'-' operator can only be used on number types, got 'bool'",
    );
}

#[test]
fn test_unary_minus_error_on_string() {
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
fn test_if_with_bool_param_pass() {
    assert_pass(
        r#"
        func f(a bool) {
            if a {
            }
        }
    "#,
    );
}

#[test]
fn test_if_with_comparison_pass() {
    assert_pass(
        r#"
        func f(a int, b int) {
            if a == b {
            }
        }
    "#,
    );
}

#[test]
fn test_if_else_pass() {
    assert_pass(
        r#"
        func f(a bool) {
            if a {
            } else {
            }
        }
    "#,
    );
}

#[test]
fn test_if_elseif_else_pass() {
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
}

#[test]
fn test_if_with_assignment_pass() {
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
fn test_if_condition_error_int_param() {
    assert_error(
        r#"
        func f(a int) {
            if a {
            }
        }
    "#,
        "expression must be of type 'bool', got 'i32'",
    );
}

#[test]
fn test_if_condition_error_int_literal() {
    assert_error(
        r#"
        func f() {
            if 1 {
            }
        }
    "#,
        "expression must be of type 'bool', got 'i32'",
    );
}

#[test]
fn test_if_condition_error_string() {
    assert_error(
        r#"
        func f(s string) {
            if s {
            }
        }
    "#,
        "expression must be of type 'bool', got 'string'",
    );
}

#[test]
fn test_if_condition_error_function_call() {
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
fn test_if_block_variable_not_visible_outside() {
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
}

#[test]
fn test_if_block_outer_variable_accessible() {
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
fn test_while_stmt_with_bool_pass() {
    assert_pass(
        r#"
        func f(a bool) {
            while a {
            }
        }
    "#,
    );
}

#[test]
fn test_while_stmt_with_comparison_pass() {
    assert_pass(
        r#"
        func f(a int, b int) {
            while a < b {
                a = a + 1
            }
        }
    "#,
    );
}

#[test]
fn test_while_condition_error_int_param() {
    assert_error(
        r#"
        func f(a int) {
            while a {
            }
        }
    "#,
        "expression must be of type 'bool', got 'i32'",
    );
}

#[test]
fn test_while_condition_error_int_literal() {
    assert_error(
        r#"
        func f() {
            while 1 {
            }
        }
    "#,
        "expression must be of type 'bool', got 'i32'",
    );
}

#[test]
fn test_while_stmt_body_is_checked() {
    assert_error(
        r#"
        func f(a bool, b int) {
            while a {
                b = true
            }
        }
    "#,
        "mismatched types in assignment. expected 'i32', got 'bool'",
    );
}

#[test]
fn test_while_does_not_satisfy_return() {
    // A return inside a while body doesn't satisfy the exhaustive return check
    assert_error(
        r#"
        func f() int {
            while true {
                return 0
            }
        }
    "#,
        "missing return in function 'f'",
    );
}

#[test]
fn test_while_nested_pass() {
    assert_pass(
        r#"
        func f(a bool, b bool) {
            while a {
                while b {
                }
            }
        }
    "#,
    );
}

#[test]
fn test_while_nested_outer_variable_accessible() {
    assert_pass(
        r#"
        func f(a bool, b bool) int {
            x := 0
            while a {
                while b {
                    x = x + 1
                }
            }
            return x
        }
    "#,
    );
}

#[test]
fn test_while_nested_inner_return_not_exhaustive() {
    assert_error(
        r#"
        func f(a bool, b bool) int {
            while a {
                while b {
                    return 0
                }
            }
        }
    "#,
        "missing return in function 'f'",
    );
}

#[test]
fn test_while_stmt_inner_var_not_visible_outside() {
    assert_error(
        r#"
        func f(a bool) int {
            while a {
                x := 0
            }
            return x
        }
    "#,
        "not declared",
    );
}

#[test]
fn test_while_stmt_outer_var_accessible_inside() {
    assert_pass(
        r#"
        func f(a bool) int {
            x := 0
            while a {
                x = 1
            }
            return x
        }
    "#,
    );
}

#[test]
fn test_break_in_loop_pass() {
    assert_pass(
        r#"
        func f(a bool) {
            while a {
                break
            }
        }
    "#,
    );
}

#[test]
fn test_continue_in_loop_pass() {
    assert_pass(
        r#"
        func f(a bool) {
            while a {
                continue
            }
        }
    "#,
    );
}

#[test]
fn test_break_continue_both_in_loop_pass() {
    assert_pass(
        r#"
        func f(a bool, b bool) {
            while a {
                if b {
                    break
                } else {
                    continue
                }
            }
        }
    "#,
    );
}

#[test]
fn test_break_continue_nested_pass() {
    assert_pass(
        r#"
        func f(a bool, b bool) {
            while a {
                while b {
                    break
                }
                continue
            }
        }
    "#,
    );
}

#[test]
fn test_break_outside_loop_error() {
    assert_error(
        r#"
        func f() {
            break
        }
    "#,
        "break cannot be used outside a loop",
    );
}

#[test]
fn test_continue_outside_loop_error() {
    assert_error(
        r#"
        func f() {
            continue
        }
    "#,
        "continue cannot be used outside a loop",
    );
}

#[test]
fn test_break_in_if_outside_loop_error() {
    // break inside an if that is not inside a loop
    assert_error(
        r#"
        func f(a bool) {
            if a {
                break
            }
        }
    "#,
        "break cannot be used outside a loop",
    );
}

#[test]
fn test_break_in_outer_loop_only() {
    // break in inner loop does not affect outer loop's in_loop state
    assert_pass(
        r#"
        func f(a bool, b bool) {
            while a {
                while b {
                    break
                }
                break
            }
        }
    "#,
    );
}

#[test]
fn test_continue_after_inner_break_pass() {
    // continue is valid in outer loop after inner loop with break
    assert_pass(
        r#"
        func f(a bool, b bool) {
            while a {
                while b {
                    continue
                }
                break
            }
        }
    "#,
    );
}

#[test]
fn test_unary_not_as_arg_pass() {
    assert_pass(
        r#"
        func consume(v bool) {}
        func f(a bool) {
            consume(!a)
        }
    "#,
    );
}

#[test]
fn test_unary_minus_as_arg_pass() {
    assert_pass(
        r#"
        func consume(v int) {}
        func f(a int) {
            consume(-a)
        }
    "#,
    );
}

#[test]
fn test_unary_not_as_int_arg_fail() {
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
fn test_nested_if_else_outer_exhaustive() {
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
}

#[test]
fn test_nested_if_else_both_branches_exhaustive() {
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

#[test]
fn test_nomangle_modifier_pass() {
    assert_pass(
        r#"
        @nomangle
        func f() {
        }
    "#,
    );
}

#[test]
fn test_inline_modifier_pass() {
    assert_pass(
        r#"
        @inline
        func f() {
        }
    "#,
    );
}

#[test]
fn test_naked_modifier_pass() {
    assert_pass(
        r#"
        @naked
        func f() {
        }
    "#,
    );
}

#[test]
fn test_two_modifiers_pass() {
    assert_pass(
        r#"
        @inline
        @naked
        func f() {
        }
    "#,
    );
}

#[test]
fn test_three_modifiers_pass() {
    assert_pass(
        r#"
        @nomangle
        @inline
        @naked
        func f() {
        }
    "#,
    );
}

#[test]
fn test_nomangle_modifier_on_extern_error() {
    assert_error(
        r#"
        @nomangle
        extern func write(fd int, s string, len int) int
    "#,
        "'nomangle' modifier is only allowed for local functions",
    );
}

#[test]
fn test_inline_modifier_on_extern_error() {
    assert_error(
        r#"
        @inline
        extern func puts(s string)
    "#,
        "'inline' modifier is only allowed for local functions",
    );
}

#[test]
fn test_alias_modifier_wrong_arg_count_error() {
    assert_error(
        r#"
        @alias foo bar
        extern func puts(s string)
    "#,
        "'alias' modifier expects exactly one argument, got 2",
    );
}

#[test]
fn test_unknown_modifier_on_func_error() {
    assert_error(
        r#"
        @unknown
        func f() {
        }
    "#,
        "unknown modifier",
    );
}

#[test]
fn test_unknown_modifier_on_extern_error() {
    assert_error(
        r#"
        @foo
        extern func puts(s string)
    "#,
        "unknown modifier",
    );
}

#[test]
fn test_modifier_does_not_affect_call() {
    // A function with a modifier can still be called normally
    assert_pass(
        r#"
        @nomangle
        func add(a int, b int) int {
            return a + b
        }

        func main() int {
            return add(1, 2)
        }
    "#,
    );
}

#[test]
fn test_modifier_on_extern_pass() {
    assert_pass(
        r#"
        @alias foo
        extern func write(fd int, s string, len int) int
    "#,
    );
}

#[test]
fn test_alias_modifier() {
    assert_pass(
        r#"
        @alias foo
        extern func write(fd int, s string, len int) int

        func f() {
            foo(1, "hello", 0)
        }
    "#,
    );
}

#[test]
fn test_for_loop() {
    assert_pass(
        r#"
        func f() {
            for i := 0; i < 10; i = i + 1 {
                f()
            }
        }
    "#,
    );
}

#[test]
fn test_for_loop_expression() {
    assert_error(
        r#"
        func f() {
            for false; 1 + 2; false {
                f()
            }
        }
    "#,
        "expression must be of type 'bool', got 'i32'",
    );
}

#[test]
fn test_shadowing_type() {
    assert_error(
        r#"
        func f() {
            int := 0
        }
    "#,
        "shadowing a type is not allowed",
    );
}

// --- Type declarations ---

#[test]
fn test_type_decl_alias_pass() {
    assert_pass(
        r#"
        type Number int

        func makeNumber() Number {
            return 0
        }
    "#,
    );
}

#[test]
fn test_type_decl_alias_compatible_with_base_in_return() {
    // Alias and base share the same TypeId, so returning one where the other
    // is expected always passes.
    assert_pass(
        r#"
        type Number int

        func makeNumber() Number {
            return 0
        }

        func consumeInt() int {
            return makeNumber()
        }
    "#,
    );
}

#[test]
fn test_type_decl_base_compatible_with_alias_in_return() {
    assert_pass(
        r#"
        type Number int

        func f(n int) Number {
            return n
        }
    "#,
    );
}

#[test]
fn test_type_decl_alias_compatible_as_param() {
    assert_pass(
        r#"
        type Number int

        func consume(n int) {}

        func f(n Number) {
            consume(n)
        }
    "#,
    );
}

#[test]
fn test_type_decl_unique_not_compatible_with_base_in_return() {
    assert_error(
        r#"
        unique type ID int

        func getInt() int { return 0 }

        func f() ID {
            return getInt()
        }
    "#,
        "incorrect return type: expected 'ID', got 'i32'",
    );
}

#[test]
fn test_type_decl_unique_not_compatible_with_base_as_param() {
    assert_error(
        r#"
        unique type ID int

        func consume(n ID) {}

        func f() {
            consume(0)
        }
    "#,
        "mismatched types in function call. expected 'ID', got 'i32'",
    );
}

#[test]
fn test_type_decl_two_unique_same_base_not_compatible() {
    assert_error(
        r#"
        unique type A int
        unique type B int

        func consumeA(n A) {}

        func f(b B) {
            consumeA(b)
        }
    "#,
        "mismatched types in function call. expected 'A', got 'B'",
    );
}

#[test]
fn test_type_decl_alias_chaining_pass() {
    // type A bool; type B A → B collapses to bool TypeId through A.
    assert_pass(
        r#"
        type A bool
        type B A

        func consumeBool(x bool) {}

        func f(b B) {
            consumeBool(b)
        }
    "#,
    );
}

#[test]
fn test_type_decl_unique_then_alias_is_same_type() {
    // type B A where A is unique → B and A share the same TypeId.
    assert_pass(
        r#"
        unique type A int
        type B A

        func consumeA(x A) {}

        func f(b B) {
            consumeA(b)
        }
    "#,
    );
}

#[test]
fn test_type_decl_alias_then_unique_not_compatible_with_base() {
    assert_error(
        r#"
        type A int
        unique type B A

        func consumeInt(n int) {}

        func f(b B) {
            consumeInt(b)
        }
    "#,
        "mismatched types in function call. expected 'i32', got 'B'",
    );
}

#[test]
fn test_type_decl_unique_then_alias_not_compatible_with_primitive() {
    // B has the same TypeId as unique A, which is not the same as int.
    assert_error(
        r#"
        unique type A int
        type B A

        func consumeInt(n int) {}

        func f(b B) {
            consumeInt(b)
        }
    "#,
        "mismatched types in function call. expected 'i32', got 'A'",
    );
}

#[test]
fn test_type_decl_duplicate_name_error() {
    assert_error(
        r#"
        type Number int
        type Number bool
    "#,
        "already declared",
    );
}

#[test]
fn test_type_decl_unknown_underlying_type_error() {
    assert_error(r#"type Foo Bar"#, "not a type");
}

#[test]
fn test_cast_overflow_u8() {
    assert_error(
        r#"func f() u8 { return 1024 as u8 }"#,
        "constant value overflows target type 'u8'",
    );
}

#[test]
fn test_cast_overflow_i8_negative() {
    assert_error(
        r#"func f() i8 { return 200 as i8 }"#,
        "constant value overflows target type 'i8'",
    );
}

#[test]
fn test_cast_overflow_u8_negative_int() {
    assert_error(
        r#"func f() u8 { return -1 as u8 }"#,
        "constant value overflows target type 'u8'",
    );
}

#[test]
fn test_cast_no_overflow_pass() {
    assert_pass(r#"func f() u8 { return 255 as u8 }"#);
}

#[test]
fn test_cast_widening_pass() {
    assert_pass(r#"func f() i64 { return 100 as i64 }"#);
}

#[test]
fn test_cast_float_to_int_overflow() {
    assert_error(
        r#"func f() u8 { return 300.0 as u8 }"#,
        "constant value overflows target type 'u8'",
    );
}

#[test]
fn test_cast_float_to_int_negative_overflow() {
    assert_error(
        r#"func f() u8 { return -1.0 as u8 }"#,
        "constant value overflows target type 'u8'",
    );
}
