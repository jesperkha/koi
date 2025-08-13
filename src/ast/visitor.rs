use crate::{
    ast::{FuncNode, ReturnNode},
    token::Token,
};

pub trait Visitor {
    fn visit_literal(&mut self, node: &Token);
    fn visit_return(&mut self, node: &ReturnNode);
    fn visit_func(&mut self, node: &FuncNode);
}
