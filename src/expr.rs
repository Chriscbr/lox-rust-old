use crate::token::TokenType;

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug)]
pub enum Expr {
    Literal(Literal),
    Unary(TokenType, Box<Expr>),
    Binary(Box<Expr>, TokenType, Box<Expr>),
    Grouping(Box<Expr>),
}

pub trait Visitor<T> {
    fn visit_expr(&mut self, e: &Expr) -> T;
}
