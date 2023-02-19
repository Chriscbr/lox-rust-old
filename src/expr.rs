use crate::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assign(Assign),
    Binary(Binary),
    Call(Call),
    Grouping(Grouping),
    Literal(Literal),
    Logical(Logical),
    Variable(Variable),
    Unary(Unary),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assign {
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: TokenKind,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub callee: Box<Expr>,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Grouping {
    pub expression: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logical {
    pub left: Box<Expr>,
    pub operator: TokenKind,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub operator: TokenKind,
    pub right: Box<Expr>,
}
