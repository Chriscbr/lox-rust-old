use std::iter::Peekable;

use crate::{
    expr::{Expr, Literal},
    token::{Token, TokenType},
};

use anyhow::anyhow;
use anyhow::Result;

pub struct Parser;
impl<'a> Parser {
    pub fn parse(&self, source: Vec<Token>) -> Result<Expr> {
        let mut iter = source.into_iter().peekable();

        Ok(self.parse_expression(&mut iter)?)
    }

    fn parse_expression<I: Iterator<Item = Token<'a>>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Expr> {
        self.parse_equality(iter)
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
            _ => {
                let token = iter.next().unwrap();
                Err(anyhow!(
                    "Expected an expression, found {} on line {}",
                    token.lexeme,
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
}
