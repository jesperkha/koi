use std::process::exit;

use koi::{
    error::ErrorSet,
    ir::{IR, print_ir},
    parser::Parser,
    scanner::Scanner,
    token::File,
    types::Checker,
};

fn print_and_exit(e: ErrorSet) {
    println!("{}", e);
    exit(1);
}

fn main() {
    let file = File::new_from_file("main.koi");
    let toks = Scanner::scan(&file).expect("Failed to scan file");

    let ast = Parser::parse(&file, toks).map_err(print_and_exit).unwrap();
    let ctx = Checker::check(&ast, &file).map_err(print_and_exit).unwrap();
    let ir = IR::emit(&ast, &ctx).map_err(print_and_exit).unwrap();

    print_ir(ir);

    // Printer::print(&ast);
}
