use std::str::CharIndices;

use anyhow::anyhow;
use anyhow::Result;
use itertools::{Itertools, MultiPeek};

use crate::token::{Token, TokenType};

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

        tokens.push(Token::new(TokenType::Eof, "", line));

        Ok(tokens)
    }

    fn scan_token(&self, iter: &mut CharIter, line: &mut u32) -> Result<Option<Token>> {
        loop {
            iter.reset_peek(); // reset the "peek" cursor

            if let Some(pair) = iter.next() {
                // in most cases we want to break and return, but if we encounter
                // a newline or comment, we continue the loop instead
                break match pair {
                    (idx, '(') => self.create_token(TokenType::LeftParen, idx, 1, line),
                    (idx, ')') => self.create_token(TokenType::RightParen, idx, 1, line),
                    (idx, '{') => self.create_token(TokenType::LeftBrace, idx, 1, line),
                    (idx, '}') => self.create_token(TokenType::RightBrace, idx, 1, line),
                    (idx, ',') => self.create_token(TokenType::Comma, idx, 1, line),
                    (idx, '.') => self.create_token(TokenType::Dot, idx, 1, line),
                    (idx, '-') => self.create_token(TokenType::Minus, idx, 1, line),
                    (idx, '+') => self.create_token(TokenType::Plus, idx, 1, line),
                    (idx, ';') => self.create_token(TokenType::Semicolon, idx, 1, line),
                    (idx, '*') => self.create_token(TokenType::Star, idx, 1, line),
                    (idx, '!') => {
                        if self.peek_match(iter, |ch| ch == '=') {
                            iter.next();
                            self.create_token(TokenType::BangEqual, idx, 2, line)
                        } else {
                            self.create_token(TokenType::Bang, idx, 1, line)
                        }
                    }
                    (idx, '=') => {
                        if self.peek_match(iter, |ch| ch == '=') {
                            iter.next();
                            self.create_token(TokenType::EqualEqual, idx, 2, line)
                        } else {
                            self.create_token(TokenType::Equal, idx, 1, line)
                        }
                    }
                    (idx, '<') => {
                        if self.peek_match(iter, |ch| ch == '=') {
                            iter.next();
                            self.create_token(TokenType::LessEqual, idx, 2, line)
                        } else {
                            self.create_token(TokenType::Less, idx, 1, line)
                        }
                    }
                    (idx, '>') => {
                        if self.peek_match(iter, |ch| ch == '=') {
                            iter.next();
                            self.create_token(TokenType::GreaterEqual, idx, 2, line)
                        } else {
                            self.create_token(TokenType::Greater, idx, 1, line)
                        }
                    }
                    (idx, '/') => {
                        if self.peek_match(iter, |ch| ch == '/') {
                            iter.next();
                            // A comment goes until the end of the line
                            self.read_to_end_of_line(iter);
                            continue;
                        } else {
                            self.create_token(TokenType::Slash, idx, 1, line)
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
    fn create_token(
        &self,
        typ: TokenType,
        idx: usize,
        len: usize,
        line: &u32,
    ) -> Result<Option<Token>> {
        Ok(Some(Token::new(typ, &self.source[idx..idx + len], *line)))
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
        let start = *line;
        let mut len = 1;
        while self.peek_match(iter, |ch| ch != '"') {
            let pair = iter.next().unwrap();
            if pair.1 == '\n' {
                *line += 1;
            }
            len += 1;
        }

        // next character is the quote
        match iter.next() {
            Some((idx, _)) => self.create_token(TokenType::String, idx - len, len + 1, &start),
            None => Err(anyhow!(
                "end of line while scanning string literal on line {}",
                start
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

        self.create_token(TokenType::Number, idx, len, line)
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
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        };

        self.create_token(typ, idx, len, line)
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
            tokens.iter().map(|tok| tok.typ).collect::<Vec<TokenType>>(),
            [
                TokenType::Bang,
                TokenType::BangEqual,
                TokenType::BangEqual,
                TokenType::Equal,
                TokenType::Eof,
            ]
        );
    }

    #[test]
    fn it_ignores_comments() {
        let scanner = Scanner::new("() // hello\n// last line");
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(
            tokens.iter().map(|tok| tok.typ).collect::<Vec<TokenType>>(),
            [TokenType::LeftParen, TokenType::RightParen, TokenType::Eof,]
        );
    }
}
