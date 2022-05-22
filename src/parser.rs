use std::iter::Peekable;

use crate::{
    expr::{Expr, Literal},
    stmt::Stmt,
    token::{Token, TokenType},
};

use anyhow::anyhow;
use anyhow::Result;

pub struct Parser;

impl<'a> Parser {
    pub fn parse(&self, source: Vec<Token>) -> Result<Vec<Stmt>> {
        let mut iter = source.into_iter().peekable();
        let mut statements = vec![];
        while self.peek_match(&mut iter, |token| token.typ != TokenType::Eof) {
            statements.push(self.parse_declaration(&mut iter)?);
        }
        Ok(statements)
    }

    fn parse_declaration<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        if self.peek_match(iter, |token| token.typ == TokenType::Var) {
            return self.parse_var_declaration(iter);
        } else {
            return self.parse_statement(iter);
        }
    }

    fn parse_statement<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        if self.peek_match(iter, |token| token.typ == TokenType::Print) {
            return self.parse_print_statement(iter);
        } else if self.peek_match(iter, |token| token.typ == TokenType::LeftBrace) {
            return Ok(Stmt::Block(self.parse_block(iter)?));
        } else {
            return self.parse_expression_statement(iter);
        }
    }

    fn parse_expression_statement<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        let line = iter.peek().unwrap().line;
        let expr = self.parse_expression(iter)?;
        if self.consume_match(iter, |token| token.typ == TokenType::Semicolon) {
            Ok(Stmt::Expression(expr))
        } else {
            Err(anyhow!("Expected ';' after value on line {}", line))
        }
    }

    fn parse_block<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Vec<Stmt>> {
        let mut statements = vec![];
        let line = iter.next().unwrap().line; // consume '{'
        while self.peek_match(iter, |token| token.typ != TokenType::RightBrace) {
            statements.push(self.parse_declaration(iter)?);
        }
        if self.consume_match(iter, |token| token.typ == TokenType::RightBrace) {
            Ok(statements)
        } else {
            Err(anyhow!("Expected '}}' to match '{{' on line {}", line))
        }
    }

    fn parse_print_statement<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        let print_line = iter.next().unwrap().line; // consume 'print'
        let value_line = iter
            .peek()
            .ok_or(anyhow!(
                "Expected value after 'print' on line {}",
                print_line
            ))?
            .line;
        let value = self.parse_expression(iter)?;
        if self.consume_match(iter, |token| token.typ == TokenType::Semicolon) {
            Ok(Stmt::Print(value))
        } else {
            Err(anyhow!("Expected ';' after value on line {}", value_line))
        }
    }

    fn parse_var_declaration<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        let var_line = iter.next().unwrap().line; // consume 'var'
        let identifier: String = iter
            .next()
            .ok_or(anyhow!(
                "Expected identifier after 'var' on line {}",
                var_line
            ))?
            .lexeme
            .to_string();
        if !self.consume_match(iter, |token| token.typ == TokenType::Equal) {
            if self.consume_match(iter, |token| token.typ == TokenType::Semicolon) {
                return Ok(Stmt::Var(identifier, None));
            } else {
                return Err(anyhow!(
                    "Expected ';' after variable declaration on line {}",
                    var_line
                ));
            }
        }
        let initializer = self.parse_expression(iter)?;
        if self.consume_match(iter, |token| token.typ == TokenType::Semicolon) {
            Ok(Stmt::Var(identifier, Some(initializer)))
        } else {
            Err(anyhow!(
                "Expected ';' after variable declaration on line {}",
                var_line
            ))
        }
    }

    fn parse_expression<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Expr> {
        self.parse_assignment(iter)
    }

    fn parse_assignment<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Expr> {
        let expr = self.parse_equality(iter)?;
        if self.consume_match(iter, |token| token.typ == TokenType::Equal) {
            let line = iter.peek().unwrap().line;
            let value = self.parse_assignment(iter)?;
            match expr {
                Expr::Variable(name) => Ok(Expr::Assign(name, Box::from(value))),
                _ => Err(anyhow!("Invalid assignment target on line {}", line)),
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_equality<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Expr> {
        let mut expr = self.parse_comparison(iter)?;
        while self.peek_match(iter, |token| {
            token.typ == TokenType::BangEqual || token.typ == TokenType::EqualEqual
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_comparison(iter)?;
            expr = Expr::Binary(Box::from(expr), operator.typ, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_comparison<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Expr> {
        let mut expr = self.parse_term(iter)?;
        while self.peek_match(iter, |token| {
            token.typ == TokenType::Greater
                || token.typ == TokenType::GreaterEqual
                || token.typ == TokenType::Less
                || token.typ == TokenType::LessEqual
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_term(iter)?;
            expr = Expr::Binary(Box::from(expr), operator.typ, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_term<I: Iterator<Item = Token<'a>>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let mut expr = self.parse_factor(iter)?;
        while self.peek_match(iter, |token| {
            token.typ == TokenType::Minus || token.typ == TokenType::Plus
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_factor(iter)?;
            expr = Expr::Binary(Box::from(expr), operator.typ, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_factor<I: Iterator<Item = Token<'a>>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let mut expr = self.parse_unary(iter)?;
        while self.peek_match(iter, |token| {
            token.typ == TokenType::Slash || token.typ == TokenType::Star
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_unary(iter)?;
            expr = Expr::Binary(Box::from(expr), operator.typ, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_unary<I: Iterator<Item = Token<'a>>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        if self.peek_match(iter, |token| {
            token.typ == TokenType::Bang || token.typ == TokenType::Minus
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_unary(iter)?;
            Ok(Expr::Unary(operator.typ, Box::from(right)))
        } else {
            self.parse_primary(iter)
        }
    }

    fn parse_primary<I: Iterator<Item = Token<'a>>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let next = iter.peek().ok_or(anyhow!("expected an expression"))?;
        match next.typ {
            TokenType::False => {
                iter.next(); // consume token
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            TokenType::True => {
                iter.next(); // consume token
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            TokenType::Nil => {
                iter.next(); // consume token
                Ok(Expr::Literal(Literal::Nil))
            }
            TokenType::Number => Ok(Expr::Literal(Literal::Number(
                iter.next() // consume token
                    .unwrap()
                    .number
                    .ok_or(anyhow!("expected number in token"))?,
            ))),
            TokenType::String => Ok(Expr::Literal(Literal::String(
                iter.next() // consume token
                    .unwrap()
                    .string // we take ownership of the string! cool
                    .ok_or(anyhow!("expected number in token"))?,
            ))),
            TokenType::LeftParen => {
                let line = iter.next().unwrap().line; // consume '('
                let expr = self.parse_expression(iter)?;
                if self.peek_match(iter, |token| token.typ == TokenType::RightParen) {
                    iter.next(); // consume ')'
                    Ok(Expr::Grouping(Box::from(expr)))
                } else {
                    Err(anyhow!("Expected ')' to match '(' on line {}", line))
                }
            }
            TokenType::Identifier => Ok(Expr::Variable(iter.next().unwrap().lexeme.to_string())),
            _ => {
                let token = iter.next().unwrap();
                Err(anyhow!(
                    "Expected an expression, found \"{}\" ({}) on line {}",
                    token.lexeme,
                    token.typ,
                    token.line
                ))
            }
        }
    }

    /// Returns true if there is another character to peek which matches the
    /// predicate, otherwise it returns false.
    fn peek_match<F, I>(&self, iter: &mut Peekable<I>, pred: F) -> bool
    where
        F: FnOnce(&Token<'a>) -> bool,
        I: Iterator<Item = Token<'a>>,
    {
        if let Some(token) = iter.peek() {
            pred(token)
        } else {
            false
        }
    }

    /// Returns true if the next character matches the predicate, otherwise it
    /// returns false. Only consumes if the match succeeds.
    fn consume_match<F, I>(&self, iter: &mut Peekable<I>, pred: F) -> bool
    where
        F: FnOnce(&Token<'a>) -> bool,
        I: Iterator<Item = Token<'a>>,
    {
        if let Some(token) = iter.peek() {
            if pred(token) {
                iter.next();
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}
