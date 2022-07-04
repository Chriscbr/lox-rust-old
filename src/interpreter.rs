use std::cell::RefCell;
use std::fmt;
use std::iter::zip;

use anyhow::anyhow;
use anyhow::Result;
use generational_arena::Arena;
use generational_arena::Index;

use crate::env::Environment;
use crate::{
    expr::Expr, expr::Literal, stmt::Stmt, token::TokenKind, visitor::ExprVisitor,
    visitor::StmtVisitor,
};

// A custom error type used to signal that a value is being returned, so
// the error should be "caught" by the nearest function call.
#[derive(Debug, Clone)]
struct ReturnValueError(RuntimeValue);

impl fmt::Display for ReturnValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<returning {}>", self.0)
    }
}

impl std::error::Error for ReturnValueError {}

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
            RuntimeValue::Callable(ast, _) => {
                if let &Stmt::Function(name, _, _) = &ast {
                    write!(f, "<fn {}>", name)
                } else {
                    Err(std::fmt::Error)
                }
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

    fn define_in_env(
        &self,
        env: &Environment,
        name: String,
        value: RuntimeValue,
    ) -> (Environment, Index) {
        let index = self.variables.borrow_mut().insert(value);
        let new_env = env.insert(name, index);
        (new_env, index)
    }

    fn update_var(&self, index: Index, value: RuntimeValue) -> Result<()> {
        if let Some(old_value) = self.variables.borrow_mut().get_mut(index) {
            *old_value = value;
            Ok(())
        } else {
            Err(anyhow!(
                "Variable #{:?} is unexpectedly not allocated a value.",
                index
            ))
        }
    }

    fn lookup_in_env(&self, env: &Environment, name: &String) -> Result<RuntimeValue> {
        let index = env
            .get(name)
            .ok_or_else(|| anyhow!("Undefined variable {}.", name))?;
        if let Some(value) = self.variables.borrow().get(index) {
            Ok(value.clone())
        } else {
            Err(anyhow!("Variable {} was unexpectedly deallocated.", name))
        }
    }

    fn invoke_function(
        &self,
        callee: RuntimeValue,
        arguments: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue> {
        if let RuntimeValue::Callable(ast, closure) = callee {
            if let Stmt::Function(_name, parameters, body) = &ast {
                if parameters.len() != arguments.len() {
                    return Err(anyhow!(
                        "Expected {} arguments but got {}.",
                        parameters.len(),
                        arguments.len()
                    ));
                }

                // construct a new environment for the lifetime of the callable
                // where the parameter variables have been assigned the values
                // of the callable arguments
                let mut invoke_env = closure.enclose();
                for (param, arg) in zip(parameters, arguments) {
                    (invoke_env, _) = self.define_in_env(&invoke_env, param.clone(), arg);
                }

                // update the environment being used to interpret statements
                let old_env = self.env.replace(invoke_env);

                // evaluate each statement within our new environment
                for sub_stmt in body {
                    if let Err(err) = self.visit_stmt(sub_stmt) {
                        match err.downcast::<ReturnValueError>() {
                            Ok(ReturnValueError(value)) => {
                                // if we are returning early, be sure to restore
                                // the old environment
                                self.env.replace(old_env);
                                return Ok(value);
                            }
                            Err(err) => return Err(err),
                        }
                    }
                }

                // restore the old environment
                self.env.replace(old_env);

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

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_stmt(&self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Block(stmts) => {
                // create an environment that will encapsulate the old one
                let new_env = self.env.borrow().enclose();

                // replace the Interpreter's current environment with the empty
                // environment, returning the old one
                let old_env = self.env.replace(new_env);

                // evaluate each statement (within our new environment)
                for sub_stmt in stmts {
                    self.visit_stmt(sub_stmt)?;
                }

                // restore the environment, discarding all of the variables
                // that were defined within the block
                self.env.replace(old_env);

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
            Stmt::Return(expr) => {
                let value = self.visit_expr(expr)?;
                Err(ReturnValueError(value).into())
            }
            Stmt::Var(name, initializer) => {
                let value = match initializer {
                    Some(expr) => Some(self.visit_expr(&expr)?),
                    None => None,
                };
                let (new_env, _) = self.define_in_env(
                    &self.env.borrow(),
                    name.clone(),
                    value.unwrap_or(RuntimeValue::Nil),
                );
                self.env.replace(new_env);
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

                // initially bind function name to "nil" value so that it exists
                // in the function's closure so that recursion works
                let (new_env, index) =
                    self.define_in_env(&self.env.borrow(), name.clone(), RuntimeValue::Nil);

                let callable = RuntimeValue::Callable(function, new_env.clone());

                // update the function name's binding to actual Callable value
                self.update_var(index, callable)?;

                // use this new environment going forward in the current scope
                self.env.replace(new_env);

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
                let index = self
                    .env
                    .borrow()
                    .get(name)
                    .ok_or_else(|| anyhow!("Undefined variable {}.", name))?;
                self.update_var(index, evaluated.clone())?;
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
                        if let RuntimeValue::Number(left_num) = left_val {
                            if let RuntimeValue::Number(right_num) = right_val {
                                return Ok(RuntimeValue::Number(left_num + right_num));
                            }
                        }

                        if let RuntimeValue::String(ref left_str) = left_val {
                            if let RuntimeValue::String(right_str) = right_val {
                                let mut new_str = left_str.clone();
                                new_str.push_str(&right_str);
                                return Ok(RuntimeValue::String(new_str));
                            }
                        }

                        Err(anyhow!(
                            "Unexpected operands for + (must be a pair of numbers or pair of strings): {}, {}",
                            left_val,
                            right_val
                        ))
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
            Expr::Variable(name) => self.lookup_in_env(&self.env.borrow(), name),
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

                let mut argument_vals = vec![];
                for arg in arguments {
                    argument_vals.push(self.visit_expr(arg)?);
                }

                self.invoke_function(callee_val, argument_vals)
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
