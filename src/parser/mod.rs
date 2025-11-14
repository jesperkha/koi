mod fileset;
mod parser;

pub use fileset::new_fileset;
pub use parser::parse;

#[cfg(test)]
mod parser_test;
