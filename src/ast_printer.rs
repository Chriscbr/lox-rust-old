use crate::{
    expr::{Expr, Literal},
    visitor::ExprVisitor,
};

pub struct AstPrinter;

impl ExprVisitor<String> for AstPrinter {
    fn visit_expr(&mut self, e: &Expr) -> String {
        match &e {
            Expr::Assign(identifier, value) => {
                format!("(set! {} {})", identifier, self.visit_expr(value))
            }
            Expr::Binary(left, operator, right) => {
                format!(
                    "({} {} {})",
                    operator,
                    self.visit_expr(left),
                    self.visit_expr(right),
                )
            }
            Expr::Grouping(expr) => {
                format!("({})", self.visit_expr(expr))
            }
            Expr::Literal(literal) => match literal {
                Literal::Number(x) => x.to_string(),
                Literal::String(x) => x.to_string(),
                Literal::Bool(x) => x.to_string(),
                Literal::Nil => String::from("nil"),
            },
            Expr::Variable(identifier) => identifier.to_string(),
            Expr::Unary(operator, right) => {
                format!("({} {})", operator, self.visit_expr(right))
            }
        }
    }
}
