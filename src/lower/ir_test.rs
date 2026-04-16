use crate::{
    ir::unit_to_string,
    util::{compare_string_lines_or_panic, emit_string, must},
};

fn expect_equal(src: &str, expect: &str) {
    let ir_str = unit_to_string(&must(emit_string(src)));
    compare_string_lines_or_panic(ir_str, expect.to_string());
}

#[test]
fn test_function_empty_return() {
    expect_equal(
        r#"
        func f() {
            return
        }
    "#,
        r#"
        func f() void
            ret void
        "#,
    );
    expect_equal(
        r#"
        func f() {
        }
    "#,
        r#"
        func f() void
            ret void
        "#,
    );
}

#[test]
fn test_function_literal_return() {
    expect_equal(
        r#"
        func f() int {
            return 0
        }
    "#,
        r#"
        func f() i32
            ret i32 0
        "#,
    );
    expect_equal(
        r#"
        func f() float {
            return 1.2
        }
    "#,
        r#"
        func f() f64
            ret f64 1.2
        "#,
    );
    expect_equal(
        r#"
        func f() bool {
            return true
        }
    "#,
        r#"
        func f() u8
            ret u8 1
        "#,
    );
}

#[test]
fn test_function_parameter_return() {
    expect_equal(
        r#"
        func f(a int) int {
            return a
        }
    "#,
        r#"
        func f(i32) i32
            ret i32 %0
        "#,
    );
}

#[test]
fn test_function_call() {
    expect_equal(
        r#"
        func f() int {
            return 0
        }

        func g() int {
            return f()
        }
    "#,
        r#"
        func f() i32
            ret i32 0

        func g() i32
            $0 i32 = call f()
            ret i32 $0
        "#,
    );
}

#[test]
fn test_function_call_with_params() {
    expect_equal(
        r#"
        func f(a int, b bool) int {
            return a
        }

        func g(a int, b bool) int {
            return f(a, b)
        }
    "#,
        r#"
        func f(i32, u8) i32
            ret i32 %0

        func g(i32, u8) i32
            $0 i32 = call f(%0 i32, %1 u8)
            ret i32 $0
        "#,
    );
}

#[test]
fn test_multiple_function_calls() {
    expect_equal(
        r#"
        func f(a int) int {
            return a
        }

        func g(a int) int {
            return f(f(f(a)))
        }
    "#,
        r#"
        func f(i32) i32
            ret i32 %0

        func g(i32) i32
            $0 i32 = call f(%0 i32)
            $1 i32 = call f($0 i32)
            $2 i32 = call f($1 i32)
            ret i32 $2
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) int {
            return f(1, f(f(2, a), f(3, a)))
        }
    "#,
        r#"
        func f(i32, i32) i32
            $0 i32 = call f(2 i32, %0 i32)
            $1 i32 = call f(3 i32, %0 i32)
            $2 i32 = call f($0 i32, $1 i32)
            $3 i32 = call f(1 i32, $2 i32)
            ret i32 $3
        "#,
    );
}

#[test]
fn test_extern() {
    expect_equal(
        r#"
        extern func write(fd int, s string, len int) int
    "#,
        r#"
        extern func write(i32, string, i32) i32
        "#,
    );
}

#[test]
fn test_variable_decl() {
    expect_equal(
        r#"
        func f() {
            a := 0
        }
    "#,
        r#"
        func f() void
            $0 i32 = 0
            ret void
        "#,
    );
    expect_equal(
        r#"
        func f() int {
            a := 0
            b :: a
            return b
        }
    "#,
        r#"
        func f() i32
            $0 i32 = 0
            $1 i32 = $0
            ret i32 $1
        "#,
    );
}

#[test]
fn test_binary_arithmetic() {
    expect_equal(
        r#"
        func f(a int, b int) int {
            return a + b
        }
    "#,
        r#"
        func f(i32, i32) i32
            $0 i32 = add %0 %1
            ret i32 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) int {
            return a - b
        }
    "#,
        r#"
        func f(i32, i32) i32
            $0 i32 = sub %0 %1
            ret i32 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) int {
            return a * b
        }
    "#,
        r#"
        func f(i32, i32) i32
            $0 i32 = mul %0 %1
            ret i32 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) int {
            return a / b
        }
    "#,
        r#"
        func f(i32, i32) i32
            $0 i32 = div %0 %1
            ret i32 $0
        "#,
    );
}

#[test]
fn test_binary_modulo() {
    // Modulo produces u32
    expect_equal(
        r#"
        func f(a int, b int) {
            c := a % b
        }
    "#,
        r#"
        func f(i32, i32) void
            $0 u32 = mod %0 %1
            $1 u32 = $0
            ret void
        "#,
    );
}

#[test]
fn test_binary_comparison() {
    // Comparisons produce u8 (bool lowers to u8)
    expect_equal(
        r#"
        func f(a int, b int) bool {
            return a == b
        }
    "#,
        r#"
        func f(i32, i32) u8
            $0 u8 = eq %0 %1
            ret u8 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) bool {
            return a != b
        }
    "#,
        r#"
        func f(i32, i32) u8
            $0 u8 = ne %0 %1
            ret u8 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) bool {
            return a < b
        }
    "#,
        r#"
        func f(i32, i32) u8
            $0 u8 = lt %0 %1
            ret u8 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) bool {
            return a > b
        }
    "#,
        r#"
        func f(i32, i32) u8
            $0 u8 = gt %0 %1
            ret u8 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) bool {
            return a <= b
        }
    "#,
        r#"
        func f(i32, i32) u8
            $0 u8 = le %0 %1
            ret u8 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) bool {
            return a >= b
        }
    "#,
        r#"
        func f(i32, i32) u8
            $0 u8 = ge %0 %1
            ret u8 $0
        "#,
    );
}

#[test]
fn test_binary_logical() {
    expect_equal(
        r#"
        func f(a bool, b bool) bool {
            return a && b
        }
    "#,
        r#"
        func f(u8, u8) u8
            $0 = cond %0 and %1
            ret u8 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a bool, b bool) bool {
            return a || b
        }
    "#,
        r#"
        func f(u8, u8) u8
            $0 = cond %0 or %1
            ret u8 $0
        "#,
    );
}

#[test]
fn test_binary_chained() {
    // a + b + c — second binary uses first's $result
    expect_equal(
        r#"
        func f(a int, b int) int {
            return a + b + a
        }
    "#,
        r#"
        func f(i32, i32) i32
            $0 i32 = add %0 %1
            $1 i32 = add $0 %0
            ret i32 $1
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int, c int, d bool) bool {
            return (a + -b) * c == 0 && d
        }
    "#,
        r#"
        func f(i32, i32, i32, u8) u8
            $4 = cond $3 and %3
                $0 i32 = neg %1
                $1 i32 = add %0 $0
                $2 i32 = mul $1 %2
                $3 u8 = eq $2 0


            ret u8 $4
        "#,
    );
}

#[test]
fn test_unary_neg() {
    expect_equal(
        r#"
        func f(a int) int {
            return -a
        }
    "#,
        r#"
        func f(i32) i32
            $0 i32 = neg %0
            ret i32 $0
        "#,
    );
    // Double negation
    expect_equal(
        r#"
        func f(a int) int {
            return --a
        }
    "#,
        r#"
        func f(i32) i32
            $0 i32 = neg %0
            $1 i32 = neg $0
            ret i32 $1
        "#,
    );
}

#[test]
fn test_unary_not() {
    // ! produces u8 (bool)
    expect_equal(
        r#"
        func f(a bool) bool {
            return !a
        }
    "#,
        r#"
        func f(u8) u8
            $0 u8 = not %0
            ret u8 $0
        "#,
    );
    // Double not
    expect_equal(
        r#"
        func f(a bool) bool {
            return !!a
        }
    "#,
        r#"
        func f(u8) u8
            $0 u8 = not %0
            $1 u8 = not $0
            ret u8 $1
        "#,
    );
}

#[test]
fn test_unary_in_binary() {
    // -a used as operand in binary expression
    expect_equal(
        r#"
        func f(a int, b int) int {
            return a + -b
        }
    "#,
        r#"
        func f(i32, i32) i32
            $0 i32 = neg %1
            $1 i32 = add %0 $0
            ret i32 $1
        "#,
    );
}

#[test]
fn test_variable_assign() {
    expect_equal(
        r#"
        func f() {
            a := 0
            a = 1
            a = 2
            b := 3
            a = b
        }
    "#,
        r#"
        func f() void
            $0 i32 = 0
            $0 i32 = 1
            $0 i32 = 2
            $1 i32 = 3
            $0 i32 = $1
            ret void
        "#,
    );
}

#[test]
fn test_if_else() {
    expect_equal(
        r#"
        func f(a bool) int {
            if a {
                return 0
            } else {
                return 1
            }
        }
    "#,
        r#"
        func f(u8) i32
            if %0
                ret i32 0
            else
                ret i32 1
            ret void
        "#,
    );
}

#[test]
fn test_if_elseif_else() {
    // Simple bool params — elseif has no cond_ins
    expect_equal(
        r#"
        func f(a bool, b bool) int {
            if a {
                return 0
            } else if b {
                return 1
            } else {
                return 2
            }
        }
    "#,
        r#"
        func f(u8, u8) i32
            if %0
                ret i32 0
            else if (
            ): %1
                ret i32 1
            else
                ret i32 2
            ret void
        "#,
    );
}

#[test]
fn test_if_elseif_else_computed() {
    // Conditions require computation — elseif has cond_ins
    expect_equal(
        r#"
        func f(a int, b int) int {
            if a > 0 {
                return 1
            } else if a < b {
                return 2
            } else {
                return 3
            }
        }
    "#,
        r#"
        func f(i32, i32) i32
            $0 u8 = gt %0 0
            if $0
                ret i32 1
            else if (
                $1 u8 = lt %0 %1
            ): $1
                ret i32 2
            else
                ret i32 3
            ret void
        "#,
    );
}

#[test]
fn test_if_elseif_no_else() {
    expect_equal(
        r#"
        func f(a bool, b bool) {
            if a {
                return
            } else if b {
                return
            }
        }
    "#,
        r#"
        func f(u8, u8) void
            if %0
                ret void
            else if (
            ): %1
                ret void
            ret void
        "#,
    );
}

#[test]
fn test_if_else_nested_in_then() {
    expect_equal(
        r#"
        func f(a bool, b bool) int {
            if a {
                if b {
                    return 0
                } else {
                    return 1
                }
            } else {
                return 2
            }
        }
    "#,
        r#"
        func f(u8, u8) i32
            if %0
                if %1
                    ret i32 0
                else
                    ret i32 1
            else
                ret i32 2
            ret void
        "#,
    );
}

#[test]
fn test_if_else_nested_in_else() {
    expect_equal(
        r#"
        func f(a bool, b bool) int {
            if a {
                return 0
            } else {
                if b {
                    return 1
                } else {
                    return 2
                }
            }
        }
    "#,
        r#"
        func f(u8, u8) i32
            if %0
                ret i32 0
            else
                if %1
                    ret i32 1
                else
                    ret i32 2
            ret void
        "#,
    );
}

#[test]
fn test_while_simple() {
    // Condition is a plain bool param — no cond_ins generated
    expect_equal(
        r#"
        func f(a bool) {
            while a {
            }
        }
    "#,
        r#"
        func f(u8) void
            while %0
            ret void
        "#,
    );
}

#[test]
fn test_while_with_body() {
    expect_equal(
        r#"
        func f(a bool) {
            while a {
                a = false
            }
        }
    "#,
        r#"
        func f(u8) void
            while %0
                %0 u8 = 0
            ret void
        "#,
    );
}

#[test]
fn test_while_nested() {
    expect_equal(
        r#"
        func f(a bool, b bool) {
            while a {
                while b {
                }
            }
        }
    "#,
        r#"
        func f(u8, u8) void
            while %0
                while %1
            ret void
        "#,
    );
}

#[test]
fn test_while_break() {
    expect_equal(
        r#"
        func f(a bool) {
            while a {
                break
            }
        }
    "#,
        r#"
        func f(u8) void
            while %0
                break
            ret void
        "#,
    );
}

#[test]
fn test_while_continue() {
    expect_equal(
        r#"
        func f(a bool) {
            while a {
                continue
            }
        }
    "#,
        r#"
        func f(u8) void
            while %0
                continue
            ret void
        "#,
    );
}

#[test]
fn test_while_break_continue_nested() {
    // break exits inner loop, continue restarts outer loop
    expect_equal(
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
        r#"
        func f(u8, u8) void
            while %0
                while %1
                    break
                continue
            ret void
        "#,
    );
}

#[test]
fn test_while_computed_condition() {
    // Condition requires computation — cond_ins are stored inside WhileIns
    // and not shown in the IR text; only the resulting cond rvalue ($0) appears.
    expect_equal(
        r#"
        func f(a int, b int) {
            while a < b {
                a = a + 1
            }
        }
    "#,
        r#"
        func f(i32, i32) void
            while $0
                $1 i32 = add %0 1
                %0 i32 = $1
            ret void
        "#,
    );
}
