use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    kind: TokenType,
    lexeme: String,
    line: usize,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String, line: usize) -> Self {
        Token { kind, lexeme, line }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
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

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords.
    Var,
    Fun,
    For,
    And,
    Else,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    While,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenError {
    UnexpectedCharacter(char, usize),
    UnterminatedString(usize),
    UnterminatedComment(usize),
    InvalidNumber(String, usize),
}

impl std::error::Error for TokenError {}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenError::UnexpectedCharacter(c, line) => {
                write!(f, "Unexpected character '{}' at line {}", c, line)
            }
            TokenError::UnterminatedString(line) => {
                write!(f, "Unterminated string at line {}", line)
            }
            TokenError::InvalidNumber(num_str, line) => {
                write!(f, "Invalid number '{}' at line {}", num_str, line)
            }
            TokenError::UnterminatedComment(line) => {
                write!(f, "Unterminated comment at line {}", line)
            }
        }
    }
}

pub type TokenResult<T> = Result<T, TokenError>;
