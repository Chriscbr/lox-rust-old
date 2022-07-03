use crate::{
    cursor::Cursor,
    expr::{Expr, Literal},
    stmt::Stmt,
    token::{Token, TokenKind},
};

use anyhow::anyhow;
use anyhow::Result;

#[derive(Debug)]
pub struct Parser {
    cursor: Cursor<Token>,
    token: Token,
    prev_token: Token,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Parser {
            cursor: Cursor::new(tokens),
            token: Token::dummy(),
            prev_token: Token::dummy(),
        };

        parser.bump();
        parser
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = vec![];
        while !self.check(&TokenKind::Eof) {
            statements.push(self.parse_declaration()?);
        }
        Ok(statements)
    }

    fn parse_declaration(&mut self) -> Result<Stmt> {
        if self.eat(&TokenKind::Fun) {
            self.parse_function()
        } else if self.eat(&TokenKind::Var) {
            self.parse_var_declaration()
        } else {
            self.parse_statement()
        }
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        if self.check(&TokenKind::For) {
            self.parse_for_statement()
        } else if self.check(&TokenKind::If) {
            self.parse_if_statement()
        } else if self.eat(&TokenKind::Print) {
            self.parse_print_statement()
        } else if self.eat(&TokenKind::Return) {
            self.parse_return_statement()
        } else if self.eat(&TokenKind::While) {
            self.parse_while_statement()
        } else if self.eat(&TokenKind::LeftBrace) {
            Ok(Stmt::Block(self.parse_block()?))
        } else {
            self.parse_expression_statement()
        }
    }

    fn parse_for_statement(&mut self) -> Result<Stmt> {
        self.expect(&TokenKind::For, "Expected 'for' statement.".into())?;
        self.expect(&TokenKind::LeftParen, "Expected '(' after 'for'.".into())?;
        let initializer = if self.check(&TokenKind::Semicolon) {
            None
        } else if self.eat(&TokenKind::Var) {
            Some(self.parse_var_declaration()?)
        } else {
            Some(self.parse_expression_statement()?)
        };
        let mut condition = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(
            &TokenKind::Semicolon,
            "Expected ';' after loop condition.".into(),
        )?;
        let increment = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(
            &TokenKind::RightParen,
            "Expected ')' after for clauses.".into(),
        )?;
        let mut body = self.parse_statement()?;
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

    fn parse_if_statement(&mut self) -> Result<Stmt> {
        self.expect(&TokenKind::If, "Expected if statement.".into())?;
        self.expect(&TokenKind::LeftParen, "Expected '(' after 'if'.".into())?;
        let condition = self.parse_expression()?;
        self.expect(
            &TokenKind::RightParen,
            "Expected ')' after condition.".into(),
        )?;

        let then_branch = self.parse_statement()?;
        if self.check(&TokenKind::Else) {
            let else_branch = self.parse_statement()?;
            Ok(Stmt::If(
                condition,
                then_branch.into(),
                Some(else_branch.into()),
            ))
        } else {
            Ok(Stmt::If(condition, then_branch.into(), None))
        }
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt> {
        let line = self.token.line;
        let expr = self.parse_expression()?;
        if self.eat(&TokenKind::Semicolon) {
            Ok(Stmt::Expression(expr))
        } else {
            Err(anyhow!("Expected ';' after value on line {}", line))
        }
    }

    fn parse_while_statement(&mut self) -> Result<Stmt> {
        let while_line = self.prev_token.line;
        self.expect(
            &TokenKind::LeftParen,
            format!("Expected '(' after 'while' on line {}.", while_line),
        )?;
        let condition = self.parse_expression()?;
        self.expect(
            &TokenKind::RightParen,
            "Expected ')' after condition.".into(),
        )?;
        let body = self.parse_statement()?;
        Ok(Stmt::While(condition, body.into()))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = vec![];
        let open_brace_line = self.prev_token.line;
        while !self.check(&TokenKind::RightBrace) {
            statements.push(self.parse_declaration()?);
        }
        if self.eat(&TokenKind::RightBrace) {
            Ok(statements)
        } else {
            Err(anyhow!(
                "Expected '}}' to match '{{' on line {}",
                open_brace_line
            ))
        }
    }

    fn parse_print_statement(&mut self) -> Result<Stmt> {
        let value_line = self.token.line;
        let value = self.parse_expression()?;
        self.expect(
            &TokenKind::Semicolon,
            format!("Expected ';' after value on line {}", value_line),
        )?;
        Ok(Stmt::Print(value))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt> {
        let value_line = self.token.line;
        let value = self.parse_expression()?;
        self.expect(
            &TokenKind::Semicolon,
            format!("Expected ';' after return value on line {}", value_line),
        )?;
        Ok(Stmt::Return(value))
    }

    fn parse_var_declaration(&mut self) -> Result<Stmt> {
        let var_line = self.prev_token.line;
        let identifier = self.expect_identifier()?;
        if !self.eat(&TokenKind::Equal) {
            if self.eat(&TokenKind::Semicolon) {
                return Ok(Stmt::Var(identifier, None));
            } else {
                return Err(anyhow!(
                    "Expected ';' after variable declaration on line {}",
                    var_line
                ));
            }
        }
        let initializer = self.parse_expression()?;
        if self.eat(&TokenKind::Semicolon) {
            Ok(Stmt::Var(identifier, Some(initializer)))
        } else {
            Err(anyhow!(
                "Expected ';' after variable declaration on line {}",
                var_line
            ))
        }
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    fn parse_function(&mut self) -> Result<Stmt> {
        let name = self.expect_identifier()?;
        self.expect(
            &TokenKind::LeftParen,
            format!("Expected '(' after {} on line {}", name, self.token.line),
        )?;
        let mut parameters = vec![];
        if !self.check(&TokenKind::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(anyhow!("Can't have more than 255 parameters."));
                }
                parameters.push(self.expect_identifier()?);
                if self.check(&TokenKind::Comma) {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        self.expect(
            &TokenKind::RightParen,
            "Expect ')' after parameters.".into(),
        )?;
        self.expect(
            &TokenKind::LeftBrace,
            format!("Expected '{{' before function body."),
        )?;
        let body = self.parse_block()?;
        Ok(Stmt::Function(name, parameters, body))
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_or()?;
        if self.eat(&TokenKind::Equal) {
            let line = self.token.line;
            let value = self.parse_assignment()?;
            match expr {
                Expr::Variable(name) => Ok(Expr::Assign(name, Box::from(value))),
                _ => Err(anyhow!("Invalid assignment target on line {}", line)),
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_and()?;
        while self.eat(&TokenKind::Or) {
            let operator = self.prev_token.kind.clone();
            let right = self.parse_term()?;
            expr = Expr::Logical(Box::from(expr), operator, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        while self.eat(&TokenKind::And) {
            let operator = self.prev_token.kind.clone();
            let right = self.parse_term()?;
            expr = Expr::Logical(Box::from(expr), operator, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;
        while self.token.is_equality() {
            let operator = self.token.kind.clone();
            self.bump();
            let right = self.parse_comparison()?;
            expr = Expr::Binary(Box::from(expr), operator, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;
        while self.token.is_comparison() {
            let operator = self.token.kind.clone();
            self.bump();
            let right = self.parse_term()?;
            expr = Expr::Binary(Box::from(expr), operator, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;
        while self.token.is_term() {
            let operator = self.token.kind.clone();
            self.bump();
            let right = self.parse_factor()?;
            expr = Expr::Binary(Box::from(expr), operator, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        while self.token.is_factor() {
            let operator = self.token.kind.clone();
            let right = self.parse_unary()?;
            expr = Expr::Binary(Box::from(expr), operator, Box::from(right))
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.token.is_unary() {
            self.bump();
            let operator = self.token.kind.clone();
            let right = self.parse_unary()?;
            Ok(Expr::Unary(operator, Box::from(right)))
        } else {
            self.parse_call()
        }
    }

    fn parse_call(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&TokenKind::LeftParen) {
                self.bump();
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = vec![];
        if !self.check(&TokenKind::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(anyhow!("Can't have more than 255 arguments."));
                }
                arguments.push(self.parse_expression()?);
                if self.check(&TokenKind::Comma) {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        self.expect(
            &TokenKind::RightParen,
            "Expected ')' after arguments.".into(),
        )?;
        Ok(Expr::Call(Box::new(callee), arguments))
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let expr = match &self.token.kind {
            TokenKind::False => Ok(Expr::Literal(Literal::Bool(false))),
            TokenKind::True => Ok(Expr::Literal(Literal::Bool(true))),
            TokenKind::Nil => Ok(Expr::Literal(Literal::Nil)),
            TokenKind::Number(value) => Ok(Expr::Literal(Literal::Number(*value))),
            TokenKind::String(value) => Ok(Expr::Literal(Literal::String(value.clone()))),
            TokenKind::LeftParen => {
                let line = self.token.line;
                let expr = self.parse_expression()?;
                self.expect(
                    &TokenKind::RightParen,
                    format!("Expected ')' to match '(' on line {}", line),
                )?;
                Ok(Expr::Grouping(Box::from(expr)))
            }
            TokenKind::Identifier(value) => Ok(Expr::Variable(value.clone())),
            _ => Err(anyhow!(
                "Expected an expression, found token {} on line {}",
                self.token.kind,
                self.token.line
            )),
        };
        self.bump();
        expr
    }

    /// Expects and consumes the token `token`. Signals an error if the next
    /// token is not `token`.
    pub fn expect(&mut self, token: &TokenKind, message: String) -> Result<()> {
        if self.token.kind == *token {
            self.bump();
            Ok(())
        } else {
            Err(anyhow!(message))
        }
    }

    /// Expects and consumes the token `token` if it is an identifier, and
    /// signals an error otherwise.
    pub fn expect_identifier(&mut self) -> Result<String> {
        let value = match &self.token.kind {
            TokenKind::Identifier(value) => Ok(value.clone()),
            _ => {
                return Err(anyhow!(
                    "Expected an identifier, found {:?} on line {}",
                    self.token.kind,
                    self.token.line
                ))
            }
        };
        self.bump();
        value
    }

    /// Consumes one token (moves the cursor forward by one).
    fn bump(&mut self) {
        let line = self.token.line;
        self.prev_token = std::mem::replace(
            &mut self.token,
            self.cursor
                .next()
                .unwrap_or(Token::new(TokenKind::Eof, line)),
        );
    }

    /// Checks if the next token is `tok`, and returns `true` if so.
    fn check(&mut self, tok: &TokenKind) -> bool {
        self.token.kind == *tok
    }

    /// Consumes the token `token` if it exists. Returns whether the given token
    /// was present.
    fn eat(&mut self, token: &TokenKind) -> bool {
        let is_present = self.check(token);
        if is_present {
            self.bump()
        }
        is_present
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_print_stmt() {
        let tokens = vec![
            Token::new(TokenKind::Print, 1),
            Token::new(TokenKind::String("one".into()), 1),
            Token::new(TokenKind::Semicolon, 1),
            Token::new(TokenKind::Eof, 2),
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();
        let expected = vec![Stmt::Print(Expr::Literal(Literal::String("one".into())))];
        assert_eq!(result, expected)
    }
}
