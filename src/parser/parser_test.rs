use crate::ast::Printer;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::token::File;
use crate::util::{compare_string_lines_or_panic, must};

fn compare_string(src: &str) {
    let file = File::new_test_file(src);
    let toks = must(Scanner::scan(&file));
    let ast = must(Parser::parse(&file, toks));

    let pstr = Printer::to_string(&ast);
    compare_string_lines_or_panic(pstr, src.to_string());
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
