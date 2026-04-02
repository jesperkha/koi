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
        func f() i64
            ret i64 0
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
        func f(i64) i64
            ret i64 %0
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
        func f() i64
            ret i64 0

        func g() i64
            $0 i64 = call f()
            ret i64 $0
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
        func f(i64, u8) i64
            ret i64 %0

        func g(i64, u8) i64
            $0 i64 = call f(%0 i64, %1 u8)
            ret i64 $0
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
        func f(i64) i64
            ret i64 %0

        func g(i64) i64
            $0 i64 = call f(%0 i64)
            $1 i64 = call f($0 i64)
            $2 i64 = call f($1 i64)
            ret i64 $2
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) int {
            return f(1, f(f(2, a), f(3, a)))
        }
    "#,
        r#"
        func f(i64, i64) i64
            $0 i64 = call f(2 i64, %0 i64)
            $1 i64 = call f(3 i64, %0 i64)
            $2 i64 = call f($0 i64, $1 i64)
            $3 i64 = call f(1 i64, $2 i64)
            ret i64 $3
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
        extern func write(i64, string, i64) i64
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
            $0 i64 = 0
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
        func f() i64
            $0 i64 = 0
            $1 i64 = $0
            ret i64 $1
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
        func f(i64, i64) i64
            $0 i64 = add %0 %1
            ret i64 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) int {
            return a - b
        }
    "#,
        r#"
        func f(i64, i64) i64
            $0 i64 = sub %0 %1
            ret i64 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) int {
            return a * b
        }
    "#,
        r#"
        func f(i64, i64) i64
            $0 i64 = mul %0 %1
            ret i64 $0
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int) int {
            return a / b
        }
    "#,
        r#"
        func f(i64, i64) i64
            $0 i64 = div %0 %1
            ret i64 $0
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
        func f(i64, i64) void
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
        func f(i64, i64) u8
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
        func f(i64, i64) u8
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
        func f(i64, i64) u8
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
        func f(i64, i64) u8
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
        func f(i64, i64) u8
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
        func f(i64, i64) u8
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
            $0 u8 = and %0 %1
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
            $0 u8 = or %0 %1
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
        func f(i64, i64) i64
            $0 i64 = add %0 %1
            $1 i64 = add $0 %0
            ret i64 $1
        "#,
    );
    expect_equal(
        r#"
        func f(a int, b int, c int, d bool) bool {
            return (a + -b) * c == 0 && d
        }
    "#,
        r#"
        func f(i64, i64, i64, u8) u8
            $0 i64 = neg %1
            $1 i64 = add %0 $0
            $2 i64 = mul $1 %2
            $3 u8 = eq $2 0
            $4 u8 = and $3 %3
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
        func f(i64) i64
            $0 i64 = neg %0
            ret i64 $0
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
        func f(i64) i64
            $0 i64 = neg %0
            $1 i64 = neg $0
            ret i64 $1
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
        func f(i64, i64) i64
            $0 i64 = neg %1
            $1 i64 = add %0 $0
            ret i64 $1
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
            $0 i64 = 0
            $0 i64 = 1
            $0 i64 = 2
            $1 i64 = 3
            $0 i64 = $1
            ret void
        "#,
    );
}
