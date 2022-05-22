use crate::{expr::Expr, stmt::Stmt};

pub trait ExprVisitor<T> {
    fn visit_expr(&mut self, e: &Expr) -> T;
}

pub trait StmtVisitor<T> {
    fn visit_stmt(&mut self, e: &Stmt) -> T;
}
