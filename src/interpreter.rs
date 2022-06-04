use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use anyhow::anyhow;
use anyhow::Result;

use crate::expr::Expr;
use crate::expr::Literal;
use crate::stmt::Stmt;
use crate::token::TokenKind;
use crate::visitor::ExprVisitor;
use crate::visitor::StmtVisitor;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl Eq for RuntimeValue {}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Number(x) => write!(f, "{}", x),
            RuntimeValue::String(x) => write!(f, "{}", x),
            RuntimeValue::Bool(x) => write!(f, "{}", x),
            RuntimeValue::Nil => write!(f, "nil"),
        }
    }
}

impl RuntimeValue {
    pub fn unwrap_number(&self, e: anyhow::Error) -> Result<f64> {
        if let RuntimeValue::Number(val) = self {
            Ok(*val)
        } else {
            Err(e)
        }
    }
}

pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, RuntimeValue>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }
}

impl Environment {
    pub fn define(&mut self, name: String, value: RuntimeValue) -> () {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &String) -> Result<RuntimeValue> {
        if let Some(value) = self.values.get(name) {
            return Ok(value.clone());
        }

        if let Some(enclosing) = &self.enclosing {
            return Ok(enclosing.get(name)?);
        } else {
            Err(anyhow!("Undefined variable {}.", name))
        }
    }

    pub fn assign(&mut self, name: String, value: RuntimeValue) -> Result<()> {
        if let Some(_) = self.values.get(&name) {
            self.values.insert(name, value);
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return Ok(enclosing.assign(name, value)?);
        } else {
            Err(anyhow!("Undefined variable {}.", name))
        }
    }
}

pub struct Interpreter {
    env: RefCell<Environment>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter {
            env: RefCell::new(Environment::default()),
        }
    }
}

impl Interpreter {
    pub fn interpret(&self, statements: &Vec<Stmt>) -> Result<()> {
        for stmt in statements {
            self.visit_stmt(stmt)?;
        }
        Ok(())
    }
}

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_stmt(&self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Block(stmts) => {
                // create an environment that will encapsulate the old one
                let new_env = Environment::default();

                // replace the Interpreter's current environment with the empty
                // environment, returning the old one
                let old_env = self.env.replace(new_env);

                // have the new environment enclose the old one so that
                // if any new variables are defined in this environment,
                // they only last as long as the block
                self.env.borrow_mut().enclosing = Some(Box::from(old_env));

                // evaluate each statement (within our updated scope)
                for sub_stmt in stmts {
                    self.visit_stmt(sub_stmt)?;
                }

                // extract the old environment out of the enclosing one - if
                // this fails, we somehow lost the old environment
                let restored = *self
                    .env
                    .borrow_mut()
                    .enclosing
                    .take()
                    .ok_or_else(|| anyhow!("unexpected missing environment"))?;

                // restore the environment (discarding all of the variables
                // that were defined within the block)
                self.env.replace(restored);

                Ok(())
            }
            Stmt::Expression(expr) => {
                self.visit_expr(expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.visit_expr(expr)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Var(name, initializer) => {
                let value = match initializer {
                    Some(expr) => Some(self.visit_expr(&expr)?),
                    None => None,
                };
                self.env
                    .borrow_mut()
                    .define(name.clone(), value.unwrap_or(RuntimeValue::Nil));
                Ok(())
            }
            Stmt::If(condition, then_branch, else_branch) => {
                if is_truthy(&self.visit_expr(condition)?) {
                    self.visit_stmt(&then_branch)?;
                } else if let Some(unwrapped) = else_branch {
                    self.visit_stmt(unwrapped)?;
                }
                Ok(())
            }
            Stmt::While(condition, body) => {
                while is_truthy(&self.visit_expr(condition)?) {
                    self.visit_stmt(body)?;
                }
                Ok(())
            }
        }
    }
}

impl ExprVisitor<Result<RuntimeValue>> for Interpreter {
    fn visit_expr(&self, expr: &Expr) -> Result<RuntimeValue> {
        match &expr {
            Expr::Assign(name, value) => {
                let evaluated = self.visit_expr(value)?;
                self.env
                    .borrow_mut()
                    .assign(name.to_owned(), evaluated.clone())?;
                Ok(evaluated)
            }
            Expr::Binary(left, operator, right) => {
                let left_val = self.visit_expr(left)?;
                let right_val = self.visit_expr(right)?;
                match operator {
                    TokenKind::Greater => {
                        let left_num = left_val
                            .unwrap_number(anyhow!("Unexpected operand before >: {}", left_val))?;
                        let right_num = right_val
                            .unwrap_number(anyhow!("Unexpected operand after >: {}", right_val))?;
                        Ok(RuntimeValue::Bool(left_num > right_num))
                    }
                    TokenKind::GreaterEqual => {
                        let left_num = left_val
                            .unwrap_number(anyhow!("Unexpected operand before >=: {}", left_val))?;
                        let right_num = right_val
                            .unwrap_number(anyhow!("Unexpected operand after >=: {}", right_val))?;
                        Ok(RuntimeValue::Bool(left_num >= right_num))
                    }
                    TokenKind::Less => {
                        let left_num = left_val
                            .unwrap_number(anyhow!("Unexpected operand before <: {}", left_val))?;
                        let right_num = right_val
                            .unwrap_number(anyhow!("Unexpected operand after <: {}", right_val))?;
                        Ok(RuntimeValue::Bool(left_num < right_num))
                    }
                    TokenKind::LessEqual => {
                        let left_num = left_val
                            .unwrap_number(anyhow!("Unexpected operand before <=: {}", left_val))?;
                        let right_num = right_val
                            .unwrap_number(anyhow!("Unexpected operand after <=: {}", right_val))?;
                        Ok(RuntimeValue::Bool(left_num <= right_num))
                    }
                    TokenKind::BangEqual => Ok(RuntimeValue::Bool(left_val != right_val)),
                    TokenKind::EqualEqual => Ok(RuntimeValue::Bool(left_val == right_val)),
                    TokenKind::Minus => {
                        let left_num = left_val
                            .unwrap_number(anyhow!("Unexpected operand before -: {}", left_val))?;
                        let right_num = right_val
                            .unwrap_number(anyhow!("Unexpected operand after -: {}", right_val))?;
                        Ok(RuntimeValue::Number(left_num - right_num))
                    }
                    TokenKind::Plus => {
                        let left_num = left_val
                            .unwrap_number(anyhow!("Unexpected operand before +: {}", left_val))?;
                        let right_num = right_val
                            .unwrap_number(anyhow!("Unexpected operand after +: {}", right_val))?;
                        Ok(RuntimeValue::Number(left_num + right_num))
                    }
                    TokenKind::Slash => {
                        let left_num = left_val
                            .unwrap_number(anyhow!("Unexpected operand before /: {}", left_val))?;
                        let right_num = right_val
                            .unwrap_number(anyhow!("Unexpected operand after /: {}", right_val))?;
                        Ok(RuntimeValue::Number(left_num / right_num))
                    }
                    TokenKind::Star => {
                        let left_num = left_val
                            .unwrap_number(anyhow!("Unexpected operand before *: {}", left_val))?;
                        let right_num = right_val
                            .unwrap_number(anyhow!("Unexpected operand after *: {}", right_val))?;
                        Ok(RuntimeValue::Number(left_num * right_num))
                    }
                    _ => Err(anyhow!("Unexpected binary operator: {}", operator)),
                }
            }
            Expr::Grouping(expr) => self.visit_expr(expr),
            Expr::Literal(literal) => match literal {
                Literal::Number(x) => Ok(RuntimeValue::Number(*x)),
                Literal::String(x) => Ok(RuntimeValue::String(x.to_owned())),
                Literal::Bool(x) => Ok(RuntimeValue::Bool(*x)),
                Literal::Nil => Ok(RuntimeValue::Nil),
            },
            Expr::Variable(name) => self.env.borrow().get(name),
            Expr::Unary(operator, value) => {
                let evaluated = self.visit_expr(value)?;
                match operator {
                    TokenKind::Bang => Ok(RuntimeValue::Bool(is_truthy(&evaluated))),
                    TokenKind::Minus => match evaluated {
                        RuntimeValue::Number(x) => Ok(RuntimeValue::Number(-x)),
                        _ => Err(anyhow!("Unexpected operand after -: {}.", evaluated)),
                    },
                    _ => Err(anyhow!("Unexpected unary operator: {}.", operator)),
                }
            }
            Expr::Logical(left, operator, right) => {
                let left_val = self.visit_expr(left)?;
                match operator {
                    TokenKind::Or => {
                        if is_truthy(&left_val) {
                            return Ok(left_val);
                        }
                    }
                    TokenKind::And => {
                        if !is_truthy(&left_val) {
                            return Ok(left_val);
                        }
                    }
                    _ => return Err(anyhow!("Unexpected logical operator: {}.", operator)),
                };
                self.visit_expr(right)
            }
        }
    }
}

fn is_truthy(value: &RuntimeValue) -> bool {
    match value {
        RuntimeValue::Number(x) => *x != 0.0,
        RuntimeValue::String(_) => true,
        RuntimeValue::Bool(x) => *x,
        RuntimeValue::Nil => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_values_equality() {
        assert_eq!(RuntimeValue::Number(3.0), RuntimeValue::Number(3.0));
        assert_eq!(RuntimeValue::Number(-0.5), RuntimeValue::Number(-0.5));
        assert_eq!(RuntimeValue::Number(0.0), RuntimeValue::Number(0.0));
        assert_ne!(RuntimeValue::Number(0.1), RuntimeValue::Number(0.2));
        assert_ne!(RuntimeValue::Number(-5.0), RuntimeValue::Number(-6.0));
    }
}
