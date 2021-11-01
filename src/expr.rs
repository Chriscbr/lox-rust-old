use crate::token::Token;

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug)]
pub enum Expr<'a> {
    Literal(Literal),
    Unary(Token<'a>, Box<Expr<'a>>),
    Binary(Box<Expr<'a>>, Token<'a>, Box<Expr<'a>>),
    Grouping(Box<Expr<'a>>),
}

pub trait Visitor<T> {
    fn visit_expr(&mut self, e: &Expr) -> T;
}
