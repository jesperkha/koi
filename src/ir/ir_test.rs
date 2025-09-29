use crate::{
    ir::print::ir_to_string, parser::Parser, scanner::Scanner, token::File, types::Checker,
};

use super::*;

// TODO: testing suite to not rewrite helpers all the time

fn expect_equal(src: &str, expect: &str) {
    let file = File::new_test_file(src);
    let toks = Scanner::scan(&file).unwrap();
    let ast = Parser::parse(&file, toks).unwrap();
    let ctx = Checker::check(&ast, &file).unwrap();
    let ir = IR::emit(&ast, &ctx)
        .map_err(|err| panic!("{}", err))
        .unwrap();

    let ir_str = ir_to_string(ir);
    let input_lines: Vec<&str> = expect.trim().split('\n').collect();
    let fmt_lines: Vec<&str> = ir_str.trim().split('\n').collect();

    assert_eq!(
        input_lines.len(),
        fmt_lines.len(),
        "number of lines must be equal"
    );

    for (i, line) in input_lines.iter().enumerate() {
        assert_eq!(line.trim(), fmt_lines.get(i).unwrap().trim());
    }
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
        {
            ret void
        }
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
        {
            ret i64 0
        }
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
        {
            ret f64 1.2
        }
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
        {
            ret u8 1
        }
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
//         {
//             ret i64 %0
//         }
//         "#,
//     );
// }
