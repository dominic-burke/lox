use std::{fmt::{self, write}, io::BufWriter};

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
    
    pub fn get_kind(&self) -> &TokenType {
        &self.kind
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
    True,
    False,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    While,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            TokenType::And => write!(f, "and"),
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Else => write!(f, "else"),
            TokenType::Equal => write!(f, "equal"),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::False => write!(f, "false"),
            TokenType::For => write!(f, "for"),
            TokenType::Fun => write!(f, "fun"),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::Identifier(id) => write!(f, "identifer: {id}"),
            TokenType::If => write!(f, "if"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::LeftParen => write!(f, "("),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::Minus => write!(f, "-"),
            TokenType::Nil => write!(f, "nil"),
            TokenType::Number(num) => write!(f, "number: {num}"),
            TokenType::Or => write!(f, "or"),
            TokenType::Plus => write!(f, "plus"),
            TokenType::Print => write!(f, "print"),
            TokenType::Return => write!(f, "return"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::RightParen => write!(f, ")"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Star => write!(f, "*"),
            TokenType::String(str) => write!(f, "string: {str}"),
            TokenType::Super => write!(f, "super"),
            TokenType::This => write!(f, "this"),
            TokenType::True => write!(f, "true"),
            TokenType::Var => write!(f, "var"),
            TokenType::While => write!(f, "while"),
        }
    }
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
