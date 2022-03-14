use crate::expr::Expr;

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Print(Expr),
    Var(String, Expr), // note: we make declaring a value required unlike original lox
}
