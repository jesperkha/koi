use std::process::exit;

use koi::{
    ast::Printer, error::ErrorSet, parser::Parser, scanner::Scanner, token::File, types::Checker,
};

fn print_and_exit(e: ErrorSet) {
    println!("{}", e);
    exit(1);
}

fn main() {
    let file = File::new_from_file("main.koi");
    let toks = Scanner::scan(&file).expect("Failed to scan file");

    let ast = Parser::parse(&file, toks).map_err(print_and_exit).unwrap();
    let _ = Checker::check(&ast, &file).map_err(print_and_exit).unwrap();
    // let _ = IR::emit(&ast, &ctx);

    Printer::print(&ast);
}
