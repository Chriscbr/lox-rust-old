use std::fmt::{Display, Formatter, Result};

use anyhow::Context;

#[derive(Debug)]
pub struct Token<'a> {
    pub typ: TokenType,
    pub lexeme: &'a str,
    pub line: u32,
    /// Only used for Number tokens.
    pub number: Option<f64>,
    /// Only used for String tokens.
    pub string: Option<String>,
}

impl<'a> Token<'a> {
    pub fn new(typ: TokenType, lexeme: &'a str, line: u32) -> Self {
        // we are not expecting errors when creating tokens, so it's simpler to
        // panic than propagate them up as a Result.
        // could rewrite this as try_new but eh.
        let number: Option<f64> = match typ {
            TokenType::Number => Some(
                lexeme
                    .parse()
                    .with_context(|| {
                        format!(
                            "expected token to be created with a number on line {}",
                            line
                        )
                    })
                    .unwrap(),
            ),
            _ => None,
        };
        let string: Option<String> = match typ {
            TokenType::String => Some(String::from(&lexeme[1..lexeme.len() - 1])),
            _ => None,
        };

        Token {
            typ,
            lexeme,
            line,
            number,
            string,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            // Single-character tokens
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Minus => write!(f, "-"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Star => write!(f, "*"),

            // One or two character tokens
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),

            // Literals
            TokenType::Identifier => write!(f, "<IDENTIFIER>"),
            TokenType::String => write!(f, "<STRING>"),
            TokenType::Number => write!(f, "<NUMBER>"),

            // Keywords
            TokenType::And => write!(f, "and"),
            TokenType::Class => write!(f, "class"),
            TokenType::Else => write!(f, "else"),
            TokenType::False => write!(f, "false"),
            TokenType::Fun => write!(f, "fun"),
            TokenType::For => write!(f, "for"),
            TokenType::If => write!(f, "if"),
            TokenType::Nil => write!(f, "nil"),
            TokenType::Or => write!(f, "or"),
            TokenType::Print => write!(f, "print"),
            TokenType::Return => write!(f, "return"),
            TokenType::Super => write!(f, "super"),
            TokenType::This => write!(f, "this"),
            TokenType::True => write!(f, "true"),
            TokenType::Var => write!(f, "var"),
            TokenType::While => write!(f, "while"),

            TokenType::Eof => write!(f, "<EOF>"),
        }
    }
}
