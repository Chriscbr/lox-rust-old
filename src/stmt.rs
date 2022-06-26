use crate::expr::Expr;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function(String, Vec<String>, Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Var(String, Option<Expr>),
    While(Expr, Box<Stmt>),
}
