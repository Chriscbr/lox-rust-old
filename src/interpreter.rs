use std::cell::RefCell;
use std::fmt;

use anyhow::anyhow;
use anyhow::Result;
use generational_arena::Arena;

use crate::env::Environment;
use crate::{
    expr::Expr, expr::Literal, stmt::Stmt, token::TokenKind, visitor::ExprVisitor,
    visitor::StmtVisitor,
};

// TODO: remove cloneable?
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Bool(bool),
    Callable(Stmt, Environment),
    Nil,
    Number(f64),
    String(String),
}

impl Eq for RuntimeValue {}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Bool(x) => write!(f, "{}", x),
            RuntimeValue::Callable(ast, _closure) => {
                let name = match &ast {
                    &Stmt::Function(name, _parameters, _body) => name,
                    _ => panic!("Unexpected function"),
                };
                write!(f, "<fn {}>", name)
            }
            RuntimeValue::Nil => write!(f, "nil"),
            RuntimeValue::Number(x) => write!(f, "{}", x),
            RuntimeValue::String(x) => write!(f, "{}", x),
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

pub struct Interpreter {
    env: RefCell<Environment>,
    variables: RefCell<Arena<RuntimeValue>>,
    stdout: RefCell<String>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter {
            env: RefCell::new(Environment::default()),
            variables: RefCell::new(Arena::new()),
            stdout: RefCell::new(String::new()),
        }
    }
}

impl Interpreter {
    pub fn interpret(&self, statements: &Vec<Stmt>) -> Result<String> {
        for stmt in statements {
            self.visit_stmt(stmt)?;
        }
        Ok(self.stdout.take())
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
                self.stdout
                    .borrow_mut()
                    .push_str(value.to_string().as_str());
                self.stdout.borrow_mut().push('\n');
                Ok(())
            }
            Stmt::Var(name, initializer) => {
                let value = match initializer {
                    Some(expr) => Some(self.visit_expr(&expr)?),
                    None => None,
                };
                self.env.borrow_mut().define(
                    &mut self.variables.borrow_mut(),
                    name.clone(),
                    value.unwrap_or(RuntimeValue::Nil),
                );
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
            Stmt::Function(name, parameters, body) => {
                let function = Stmt::Function(name.clone(), parameters.clone(), body.clone());

                // TODO: sanity check if this makes sense?
                let callable = RuntimeValue::Callable(function, self.env.borrow().clone());
                self.env.borrow_mut().define(
                    &mut self.variables.borrow_mut(),
                    name.clone(),
                    callable,
                );
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
                self.env.borrow_mut().assign(
                    &mut self.variables.borrow_mut(),
                    name.to_owned(),
                    evaluated.clone(),
                )?;
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
            Expr::Variable(name) => self.env.borrow().get(&self.variables.borrow_mut(), name),
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
            Expr::Call(callee, arguments) => {
                let callee_val = self.visit_expr(callee)?;

                let mut argument_vals: Vec<RuntimeValue> = vec![];
                for arg in arguments {
                    argument_vals.push(self.visit_expr(arg)?);
                }

                // TODO: extract this to a function

                if let RuntimeValue::Callable(ast, closure) = callee_val {
                    if let Stmt::Function(_name, parameters, _body) = &ast {
                        if parameters.len() != argument_vals.len() {
                            return Err(anyhow!(
                                "Expected {} arguments but got {}.",
                                parameters.len(),
                                argument_vals.len()
                            ));
                        }

                        let mut environment = Environment::default();
                        environment.enclosing = Some(Box::new(closure));
                        for (param, arg) in std::iter::zip(parameters, argument_vals) {
                            environment.define(
                                &mut self.variables.borrow_mut(),
                                param.clone(),
                                arg,
                            );
                        }

                        // TODO: execute code block with the environment
                        Ok(RuntimeValue::Nil)
                    } else {
                        Err(anyhow!(
                            "Compiler error: invalid function found in callable."
                        ))
                    }
                } else {
                    Err(anyhow!("Can only call functions and classes."))
                }
            }
        }
    }
}

fn is_truthy(value: &RuntimeValue) -> bool {
    match value {
        RuntimeValue::Bool(x) => *x,
        RuntimeValue::Callable(_, _) => true,
        RuntimeValue::Nil => false,
        RuntimeValue::Number(x) => *x != 0.0,
        RuntimeValue::String(_) => true,
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
