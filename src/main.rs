use koi::{ast::Printer, parser::Parser, scanner::Scanner, token::File, types::Checker};

fn main() {
    let file = File::new_from_file("main.koi");
    let toks = Scanner::scan(&file).expect("Failed to scan file");

    let ast = Parser::parse(&file, toks)
        .map_err(|errors| {
            for error in errors {
                println!("{}", error);
            }
            std::process::exit(1);
        })
        .unwrap();

    Checker::check(&ast, &file).map_or_else(
        |errs| {
            for err in errs {
                println!("{}", err)
            }
        },
        |_| println!("check ok"),
    );

    let mut printer = Printer::new();
    printer.print(ast);
}
