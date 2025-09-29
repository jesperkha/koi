use koi::{
    ast::Printer, error::print_and_exit, ir::IR, parser::Parser, scanner::Scanner, token::File,
    types::Checker,
};

fn main() {
    let file = File::new_from_file("main.koi");
    let toks = Scanner::scan(&file).expect("Failed to scan file");

    let ast = Parser::parse(&file, toks).map_err(print_and_exit).unwrap();
    let _ = Checker::check(&ast, &file).map_err(print_and_exit).unwrap();
    // let _ = IR::emit(&ast, &ctx);

    let mut printer = Printer::new();
    printer.print(ast);
}
