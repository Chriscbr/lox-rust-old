use crate::expr::{Expr, Literal, Visitor};

pub struct AstPrinter;
impl Visitor<String> for AstPrinter {
    fn visit_expr(&mut self, e: &Expr) -> String {
        match &e {
            Expr::Literal(literal) => match literal {
                Literal::Number(x) => x.to_string(),
                Literal::String(x) => x.to_string(),
                Literal::Bool(x) => x.to_string(),
                Literal::Nil => String::from("nil"),
            },
            Expr::Unary(operator, right) => {
                format!("({} {})", operator.lexeme, self.visit_expr(right))
            }
            Expr::Binary(left, operator, right) => {
                format!(
                    "({} {} {})",
                    operator.lexeme,
                    self.visit_expr(left),
                    self.visit_expr(right),
                )
            }
            Expr::Grouping(expr) => {
                format!("({})", self.visit_expr(expr))
            }
        }
    }
}
