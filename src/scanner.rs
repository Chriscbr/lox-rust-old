use std::str::CharIndices;

use anyhow::Result;
use anyhow::{anyhow, Context};
use itertools::{Itertools, MultiPeek};

use crate::token::{Token, TokenKind};

// TODO: refactor scanner logic to use the "Cursor" class?

type CharIter<'a> = MultiPeek<CharIndices<'a>>;

pub struct Scanner<'a> {
    source: &'a str,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner { source }
    }

    pub fn scan_tokens(&self) -> Result<Vec<Token>> {
        let mut iter = self.source.char_indices().multipeek();
        let mut tokens: Vec<Token> = vec![];
        let mut line: u32 = 1;

        while let Some(token) = self.scan_token(&mut iter, &mut line)? {
            tokens.push(token);
        }

        tokens.push(Token::new(TokenKind::Eof, line));

        Ok(tokens)
    }

    fn scan_token(&self, iter: &mut CharIter, line: &mut u32) -> Result<Option<Token>> {
        loop {
            iter.reset_peek(); // reset the "peek" cursor

            if let Some(pair) = iter.next() {
                // in most cases we want to break and return, but if we encounter
                // a newline or comment, we continue the loop instead
                break match pair {
                    (_, '(') => self.create_token(TokenKind::LeftParen, line),
                    (_, ')') => self.create_token(TokenKind::RightParen, line),
                    (_, '{') => self.create_token(TokenKind::LeftBrace, line),
                    (_, '}') => self.create_token(TokenKind::RightBrace, line),
                    (_, ',') => self.create_token(TokenKind::Comma, line),
                    (_, '.') => self.create_token(TokenKind::Dot, line),
                    (_, '-') => self.create_token(TokenKind::Minus, line),
                    (_, '+') => self.create_token(TokenKind::Plus, line),
                    (_, ';') => self.create_token(TokenKind::Semicolon, line),
                    (_, '*') => self.create_token(TokenKind::Star, line),
                    (_, '!') => {
                        if self.peek_match(iter, |ch| ch == '=') {
                            iter.next();
                            self.create_token(TokenKind::BangEqual, line)
                        } else {
                            self.create_token(TokenKind::Bang, line)
                        }
                    }
                    (_, '=') => {
                        if self.peek_match(iter, |ch| ch == '=') {
                            iter.next();
                            self.create_token(TokenKind::EqualEqual, line)
                        } else {
                            self.create_token(TokenKind::Equal, line)
                        }
                    }
                    (_, '<') => {
                        if self.peek_match(iter, |ch| ch == '=') {
                            iter.next();
                            self.create_token(TokenKind::LessEqual, line)
                        } else {
                            self.create_token(TokenKind::Less, line)
                        }
                    }
                    (_, '>') => {
                        if self.peek_match(iter, |ch| ch == '=') {
                            iter.next();
                            self.create_token(TokenKind::GreaterEqual, line)
                        } else {
                            self.create_token(TokenKind::Greater, line)
                        }
                    }
                    (_, '/') => {
                        if self.peek_match(iter, |ch| ch == '/') {
                            iter.next();
                            // A comment goes until the end of the line
                            self.read_to_end_of_line(iter);
                            continue;
                        } else {
                            self.create_token(TokenKind::Slash, line)
                        }
                    }
                    (_, '"') => self.parse_string(iter, line),
                    (_, ' ' | '\r' | '\t') => continue,
                    (_, '\n') => {
                        *line += 1;
                        continue;
                    }
                    (idx, char) => {
                        if char.is_ascii_digit() {
                            self.parse_number(iter, idx, line)
                        } else if char.is_ascii_alphabetic() || char == '_' {
                            self.parse_identifer(iter, idx, line)
                        } else {
                            Err(anyhow!("unexpected character {:?} on line {}", char, line))
                        }
                    }
                };
            } else {
                // No more tokens left.
                return Ok(None);
            }
        }
    }

    // helper method
    fn create_token(&self, typ: TokenKind, line: &u32) -> Result<Option<Token>> {
        Ok(Some(Token::new(typ, *line)))
    }

    /// Returns true if there is another character to peek which matches the
    /// predicate, otherwise it returns false.
    fn peek_match<F>(&self, iter: &mut CharIter, pred: F) -> bool
    where
        F: FnOnce(char) -> bool,
    {
        if let Some(pair) = iter.peek() {
            pred(pair.1)
        } else {
            false
        }
    }

    fn read_to_end_of_line(&self, iter: &mut CharIter) -> () {
        while self.peek_match(iter, |ch| ch != '\n') {
            iter.next();
        }
    }

    fn parse_string(&self, iter: &mut CharIter, line: &mut u32) -> Result<Option<Token>> {
        let mut lexeme = String::new();
        while self.peek_match(iter, |ch| ch != '"') {
            let (_, char) = iter.next().unwrap();
            if char == '\n' {
                *line += 1;
            }
            lexeme.push(char);
        }

        // next character is the quote
        match iter.next() {
            Some(_) => self.create_token(TokenKind::String(lexeme), line),
            None => Err(anyhow!(
                "end of line while scanning string literal on line {}",
                line
            )),
        }
    }

    fn parse_number(
        &self,
        iter: &mut CharIter,
        idx: usize,
        line: &mut u32,
    ) -> Result<Option<Token>> {
        let mut len = 1;
        while self.peek_match(iter, |ch| ch.is_ascii_digit()) {
            iter.next();
            len += 1;
        }

        // Look for a fractional part
        iter.reset_peek();
        if matches!(iter.peek(), Some((_, '.'))) {
            if matches!(iter.peek(), Some((_, '0'..='9'))) {
                // consume the ".", reset peek lookahead
                iter.next();
                len += 1;

                while self.peek_match(iter, |ch| ch.is_ascii_digit()) {
                    iter.next();
                    len += 1;
                }
            }
        }

        let value: f64 = self.source[idx..idx + len]
            .parse()
            .with_context(|| format!("unable to parse number on line {}", line))
            .unwrap();
        self.create_token(TokenKind::Number(value), line)
    }

    fn parse_identifer(
        &self,
        iter: &mut CharIter,
        idx: usize,
        line: &mut u32,
    ) -> Result<Option<Token>> {
        let mut len = 1;
        while self.peek_match(iter, |ch| ch.is_alphanumeric() || ch == '_') {
            iter.next();
            len += 1;
        }

        let typ = match &self.source[idx..idx + len] {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier(self.source[idx..idx + len].to_owned()),
        };

        self.create_token(typ, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_next_resets_peek_lookahead() {
        let s = String::from("abcdef");
        let mut iter = s.chars().multipeek();
        assert_eq!(iter.next(), Some('a'));
        assert_eq!(iter.peek(), Some(&'b'));
        assert_eq!(iter.peek(), Some(&'c'));
        assert_eq!(iter.peek(), Some(&'d'));
        assert_eq!(iter.next(), Some('b'));
        assert_eq!(iter.peek(), Some(&'c'));
    }

    #[test]
    fn it_parses_characters_with_single_lookahead() {
        let scanner = Scanner::new("!!=!==");
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(
            tokens
                .iter()
                .map(|tok| tok.kind.clone())
                .collect::<Vec<TokenKind>>(),
            [
                TokenKind::Bang,
                TokenKind::BangEqual,
                TokenKind::BangEqual,
                TokenKind::Equal,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn it_ignores_comments() {
        let scanner = Scanner::new("() // hello\n// last line");
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(
            tokens
                .iter()
                .map(|tok| tok.kind.clone())
                .collect::<Vec<TokenKind>>(),
            [TokenKind::LeftParen, TokenKind::RightParen, TokenKind::Eof,]
        );
    }
}
