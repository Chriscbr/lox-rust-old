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
    Assign(String, Box<Expr>),
    Binary(Box<Expr>, TokenType, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Variable(String),
    Unary(TokenType, Box<Expr>),
}
