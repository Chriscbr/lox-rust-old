use crate::token::TokenKind;

#[derive(Debug, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Assign(String, Box<Expr>),
    Binary(Box<Expr>, TokenKind, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Logical(Box<Expr>, TokenKind, Box<Expr>),
    Variable(String),
    Unary(TokenKind, Box<Expr>),
}
