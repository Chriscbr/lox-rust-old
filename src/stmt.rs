use crate::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(Block),
    Expression(Expression),
    Function(Function),
    If(If),
    Print(Print),
    Return(Return),
    Var(Var),
    While(While),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub expression: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Print {
    pub expression: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Return {
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub name: String,
    pub initializer: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct While {
    pub condition: Expr,
    pub body: Box<Stmt>,
}
