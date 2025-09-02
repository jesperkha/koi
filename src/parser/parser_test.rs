use super::*;
use crate::ast::Printer;
use crate::scanner::Scanner;
use crate::token::File;

fn parse_string(src: &str) -> ParserResult {
    let file = File::new_test_file(src);
    let res = Scanner::new(&file).scan();
    Parser::new(&file, res.unwrap()).parse()
}

fn compare_string(src: &str) {
    let ast = parse_string(src).expect("failed to parse valid source");
    let pstr = Printer::new().to_string(ast);

    let input_lines: Vec<&str> = src.trim().split('\n').collect();
    let fmt_lines: Vec<&str> = pstr.trim().split('\n').collect();
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
fn test_function_with_return() {
    compare_string(
        r#"
        func main() {
            return
        }
    "#,
    );

    compare_string(
        r#"
        func main() int {
            return 0
        }
    "#,
    );

    compare_string(
        r#"
        func main(a int) int {
            return 0
        }
    "#,
    );

    compare_string(
        r#"
        func main(a int, b bool, c float) int {
            return 0
        }
    "#,
    );
}
