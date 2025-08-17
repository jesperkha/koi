use koi::{parser::Parser, scanner::Scanner, token::File};

fn main() {
    let file = File::new_from_file("main.koi");
    let toks = Scanner::new(&file).scan().expect("Failed to scan file");

    let mut parser = Parser::new(&file, toks);
    let ast = parser
        .parse()
        .map_err(|errors| {
            for error in errors {
                println!("{}", error);
            }
            std::process::exit(1);
        })
        .unwrap();

    println!("Parsed AST: {:#?}", ast);
}
