use koi::{
    scanner::Scanner,
    token::{File, display_tokens},
};

fn main() {
    let file = File::new_from_file("main.koi");
    let mut s = Scanner::new(&file);

    match s.scan() {
        Ok(toks) => println!("{}", display_tokens(&toks)),
        Err(e) => println!("{}", e),
    };
}
