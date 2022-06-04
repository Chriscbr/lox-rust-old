use std::iter::Peekable;

use crate::{
    expr::{Expr, Literal},
    stmt::Stmt,
    token::{Token, TokenKind},
};

use anyhow::anyhow;
use anyhow::Result;

// #[derive(Clone)]
// pub struct Cursor {
//     pub stream: Vec<Token>,
//     index: usize,
// }

// impl Iterator for Cursor {
//     type Item = TokenTree;

//     fn next(&mut self) -> Option<TokenTree> {
//         self.next_with_spacing().map(|(tree, _)| tree)
//     }
// }

// impl Cursor {
//     fn new(stream: TokenStream) -> Self {
//         Cursor { stream, index: 0 }
//     }

//     #[inline]
//     pub fn next_with_spacing(&mut self) -> Option<TreeAndSpacing> {
//         self.stream.0.get(self.index).map(|tree| {
//             self.index += 1;
//             tree.clone()
//         })
//     }

//     #[inline]
//     pub fn next_with_spacing_ref(&mut self) -> Option<&TreeAndSpacing> {
//         self.stream.0.get(self.index).map(|tree| {
//             self.index += 1;
//             tree
//         })
//     }

//     pub fn index(&self) -> usize {
//         self.index
//     }

//     pub fn append(&mut self, new_stream: TokenStream) {
//         if new_stream.is_empty() {
//             return;
//         }
//         let index = self.index;
//         let stream = mem::take(&mut self.stream);
//         *self = TokenStream::from_streams(smallvec![stream, new_stream]).into_trees();
//         self.index = index;
//     }

//     pub fn look_ahead(&self, n: usize) -> Option<&TokenTree> {
//         self.stream.0[self.index..].get(n).map(|(tree, _)| tree)
//     }
// }

pub struct Parser;

impl<'a> Parser {
    pub fn parse(&self, source: Vec<Token>) -> Result<Vec<Stmt>> {
        let mut iter = source.into_iter().peekable();
        let mut statements = vec![];
        while self.peek_match(&mut iter, |token| token.kind != TokenKind::Eof) {
            statements.push(self.parse_declaration(&mut iter)?);
        }
        Ok(statements)
    }

    fn parse_declaration<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Stmt> {
        if self.peek_match(iter, |token| token.kind == TokenKind::Var) {
            return self.parse_var_declaration(iter);
        } else {
            return self.parse_statement(iter);
        }
    }

    fn parse_statement<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Stmt> {
        if self.peek_match(iter, |token| token.kind == TokenKind::For) {
            return self.parse_for_statement(iter);
        } else if self.peek_match(iter, |token| token.kind == TokenKind::If) {
            return self.parse_if_statement(iter);
        } else if self.peek_match(iter, |token| token.kind == TokenKind::Print) {
            return self.parse_print_statement(iter);
        } else if self.peek_match(iter, |token| token.kind == TokenKind::While) {
            return self.parse_while_statement(iter);
        } else if self.peek_match(iter, |token| token.kind == TokenKind::LeftBrace) {
            return Ok(Stmt::Block(self.parse_block(iter)?));
        } else {
            return self.parse_expression_statement(iter);
        }
    }

    fn parse_for_statement<I: Iterator<Item = Token>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        iter.next(); // consume 'for'
        self.consume(iter, TokenKind::LeftParen, "Expected '(' after 'for'.")?;

        let initializer = if let Some(token) = iter.peek() {
            match token.kind {
                TokenKind::Semicolon => None,
                TokenKind::Var => Some(self.parse_var_declaration(iter)?),
                _ => Some(self.parse_expression_statement(iter)?),
            }
        } else {
            return Err(anyhow!("Expected initializer after '('."));
        };

        let mut condition = if self.peek_match(iter, |token| token.kind != TokenKind::Semicolon) {
            Some(self.parse_expression(iter)?)
        } else {
            None
        };

        self.consume(
            iter,
            TokenKind::Semicolon,
            "Expected ';' after loop condition.",
        )?;

        let increment = if self.peek_match(iter, |token| token.kind != TokenKind::Semicolon) {
            Some(self.parse_expression(iter)?)
        } else {
            None
        };

        self.consume(
            iter,
            TokenKind::RightParen,
            "Expected ')' after for clauses.",
        )?;

        let mut body = self.parse_statement(iter)?;

        if let Some(expr) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(expr).into()]);
        }

        if condition.is_none() {
            condition = Some(Expr::Literal(Literal::Bool(true)));
        }

        body = Stmt::While(condition.unwrap(), body.into());

        if let Some(expr) = initializer {
            body = Stmt::Block(vec![expr, body]);
        }

        Ok(body)
    }

    fn parse_if_statement<I: Iterator<Item = Token>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        iter.next(); // consume 'if'
        self.consume(iter, TokenKind::LeftParen, "Expected '(' after 'if'.")?;
        let condition = self.parse_expression(iter)?;
        self.consume(iter, TokenKind::RightParen, "Expected ')' after condition.")?;

        let then_branch = self.parse_statement(iter)?;
        if self.peek_match(iter, |token| token.kind == TokenKind::Else) {
            let else_branch = self.parse_statement(iter)?;
            Ok(Stmt::If(
                condition,
                then_branch.into(),
                Some(else_branch.into()),
            ))
        } else {
            Ok(Stmt::If(condition, then_branch.into(), None))
        }
    }

    fn parse_expression_statement<I: Iterator<Item = Token>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        let line = iter.peek().unwrap().line;
        let expr = self.parse_expression(iter)?;
        if self.consume_match(iter, |token| token.kind == TokenKind::Semicolon) {
            Ok(Stmt::Expression(expr))
        } else {
            Err(anyhow!("Expected ';' after value on line {}", line))
        }
    }

    fn parse_while_statement<I: Iterator<Item = Token>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        iter.next(); // consume 'while'
        self.consume(iter, TokenKind::LeftParen, "Expected '(' after 'while'.")?;
        let condition = self.parse_expression(iter)?;
        self.consume(iter, TokenKind::RightParen, "Expected ')' after condition.")?;
        let body = self.parse_statement(iter)?;
        Ok(Stmt::While(condition, body.into()))
    }

    fn parse_block<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Vec<Stmt>> {
        let mut statements = vec![];
        let line = iter.next().unwrap().line; // consume '{'
        while self.peek_match(iter, |token| token.kind != TokenKind::RightBrace) {
            statements.push(self.parse_declaration(iter)?);
        }
        if self.consume_match(iter, |token| token.kind == TokenKind::RightBrace) {
            Ok(statements)
        } else {
            Err(anyhow!("Expected '}}' to match '{{' on line {}", line))
        }
    }

    fn parse_print_statement<I: Iterator<Item = Token>>(
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
        if self.consume_match(iter, |token| token.kind == TokenKind::Semicolon) {
            Ok(Stmt::Print(value))
        } else {
            Err(anyhow!("Expected ';' after value on line {}", value_line))
        }
    }

    fn parse_var_declaration<I: Iterator<Item = Token>>(
        &self,
        iter: &mut Peekable<I>,
    ) -> Result<Stmt> {
        let var_line = iter.next().unwrap().line;
        let next = iter.next().ok_or(anyhow!("expected an identifier"))?;
        let identifier = match next.kind {
            TokenKind::Identifier(value) => value,
            _ => {
                return Err(anyhow!(
                    "Expected an expression, found {:?} on line {}",
                    next.kind,
                    next.line
                ))
            }
        };
        if !self.consume_match(iter, |token| token.kind == TokenKind::Equal) {
            if self.consume_match(iter, |token| token.kind == TokenKind::Semicolon) {
                return Ok(Stmt::Var(identifier, None));
            } else {
                return Err(anyhow!(
                    "Expected ';' after variable declaration on line {}",
                    var_line
                ));
            }
        }
        let initializer = self.parse_expression(iter)?;
        if self.consume_match(iter, |token| token.kind == TokenKind::Semicolon) {
            Ok(Stmt::Var(identifier, Some(initializer)))
        } else {
            Err(anyhow!(
                "Expected ';' after variable declaration on line {}",
                var_line
            ))
        }
    }

    fn parse_expression<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        self.parse_assignment(iter)
    }

    fn parse_assignment<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let expr = self.parse_or(iter)?;
        if self.consume_match(iter, |token| token.kind == TokenKind::Equal) {
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

    fn parse_or<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let mut expr = self.parse_and(iter)?;
        while self.peek_match(iter, |token| token.kind == TokenKind::Or) {
            let operator = iter.next().unwrap();
            let right = self.parse_term(iter)?;
            expr = Expr::Logical(Box::from(expr), operator.kind, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_and<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let mut expr = self.parse_equality(iter)?;
        while self.peek_match(iter, |token| token.kind == TokenKind::And) {
            let operator = iter.next().unwrap();
            let right = self.parse_term(iter)?;
            expr = Expr::Logical(Box::from(expr), operator.kind, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_equality<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let mut expr = self.parse_comparison(iter)?;
        while self.peek_match(iter, |token| {
            token.kind == TokenKind::BangEqual || token.kind == TokenKind::EqualEqual
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_comparison(iter)?;
            expr = Expr::Binary(Box::from(expr), operator.kind, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_comparison<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let mut expr = self.parse_term(iter)?;
        while self.peek_match(iter, |token| {
            token.kind == TokenKind::Greater
                || token.kind == TokenKind::GreaterEqual
                || token.kind == TokenKind::Less
                || token.kind == TokenKind::LessEqual
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_term(iter)?;
            expr = Expr::Binary(Box::from(expr), operator.kind, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_term<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let mut expr = self.parse_factor(iter)?;
        while self.peek_match(iter, |token| {
            token.kind == TokenKind::Minus || token.kind == TokenKind::Plus
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_factor(iter)?;
            expr = Expr::Binary(Box::from(expr), operator.kind, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_factor<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let mut expr = self.parse_unary(iter)?;
        while self.peek_match(iter, |token| {
            token.kind == TokenKind::Slash || token.kind == TokenKind::Star
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_unary(iter)?;
            expr = Expr::Binary(Box::from(expr), operator.kind, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_unary<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        if self.peek_match(iter, |token| {
            token.kind == TokenKind::Bang || token.kind == TokenKind::Minus
        }) {
            let operator = iter.next().unwrap();
            let right = self.parse_unary(iter)?;
            Ok(Expr::Unary(operator.kind, Box::from(right)))
        } else {
            self.parse_primary(iter)
        }
    }

    fn parse_primary<I: Iterator<Item = Token>>(&self, iter: &mut Peekable<I>) -> Result<Expr> {
        let next = iter.next().ok_or(anyhow!("expected an expression"))?;
        match next.kind {
            TokenKind::False => Ok(Expr::Literal(Literal::Bool(false))),
            TokenKind::True => Ok(Expr::Literal(Literal::Bool(true))),
            TokenKind::Nil => Ok(Expr::Literal(Literal::Nil)),
            TokenKind::Number(value) => Ok(Expr::Literal(Literal::Number(value))),
            TokenKind::String(value) => Ok(Expr::Literal(Literal::String(value))),
            TokenKind::LeftParen => {
                let line = next.line;
                let expr = self.parse_expression(iter)?;
                if self.peek_match(iter, |token| token.kind == TokenKind::RightParen) {
                    Ok(Expr::Grouping(Box::from(expr)))
                } else {
                    Err(anyhow!("Expected ')' to match '(' on line {}", line))
                }
            }
            TokenKind::Identifier(value) => Ok(Expr::Variable(value)),
            _ => Err(anyhow!(
                "Expected an expression, found {:?} on line {}",
                next.kind,
                next.line
            )),
        }
    }

    /// Returns true if there is another character to peek which matches the
    /// predicate, otherwise it returns false.
    fn peek_match<F, I>(&self, iter: &mut Peekable<I>, pred: F) -> bool
    where
        F: FnOnce(&Token) -> bool,
        I: Iterator<Item = Token>,
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
        F: FnOnce(&Token) -> bool,
        I: Iterator<Item = Token>,
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

    fn consume<I>(
        &self,
        iter: &mut Peekable<I>,
        kind: TokenKind,
        message: &'static str,
    ) -> Result<()>
    where
        I: Iterator<Item = Token>,
    {
        if let Some(token) = iter.peek() {
            if token.kind == kind {
                iter.next();
                Ok(())
            } else {
                Err(anyhow!(message))
            }
        } else {
            Err(anyhow!(message))
        }
    }
}
