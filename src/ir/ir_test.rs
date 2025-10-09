use crate::{
    config::Config,
    ir::print::ir_to_string,
    parser::Parser,
    scanner::Scanner,
    token::Source,
    types::Checker,
    util::{compare_string_lines_or_panic, must},
};

use super::*;

fn expect_equal(src: &str, expect: &str) {
    let config = Config::test();
    let src = Source::new_from_string(src);
    let toks = must(Scanner::scan(&src));
    let file = must(Parser::parse(src, toks, &config));
    let ctx = must(Checker::check(&file));
    let ir = must(IR::emit(&file, &ctx));

    let ir_str = ir_to_string(ir);
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

// #[test]
// fn test_function_parameter_return() {
//     expect_equal(
//         r#"
//         func f(a int) int {
//             return a
//         }
//     "#,
//         r#"
//         func f(i64) i64
//             ret i64 %0
//         "#,
//     );
// }
