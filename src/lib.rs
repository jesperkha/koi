mod ast;
mod build;
mod cmd;
mod config;
mod context;
mod driver;
mod error;
mod imports;
mod ir;
mod lower;
mod module;
mod parser;
mod scanner;
mod typecheck;
mod types;
mod util;

pub use cmd::run;
