use crate::{expr::Expr, stmt::Stmt};

// TODO: add ExprVisitorMut and StmtVisitorMut?
// which would accept "&mut self"

pub trait ExprVisitor<T> {
    fn visit_expr(&self, e: &Expr) -> T;
}

pub trait StmtVisitor<T> {
    fn visit_stmt(&self, e: &Stmt) -> T;
}
