use crate::token::{Token, TokenError, TokenResult, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}
impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            if let Err(e) = self.scan_token() {
                eprintln!("Error: {}", e);
            }
        }
        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> Option<char> {
        match self.source.chars().nth(self.current) {
            Some(c) => {
                self.current += 1;
                Some(c)
            }
            None => None,
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if let Some(c) = self.source.chars().nth(self.current) {
            if c != expected {
                return false;
            }
        } else {
            return false;
        }
        self.current += 1;
        true
    }

    fn scan_token(&mut self) -> TokenResult<()> {
        let c = self.advance();
        match c {
            Some('(') => Ok(self.add_token(TokenType::LeftParen)),
            Some(')') => Ok(self.add_token(TokenType::RightParen)),
            Some('{') => Ok(self.add_token(TokenType::LeftBrace)),
            Some('}') => Ok(self.add_token(TokenType::RightBrace)),
            Some(',') => Ok(self.add_token(TokenType::Comma)),
            Some('.') => Ok(self.add_token(TokenType::Dot)),
            Some('-') => Ok(self.add_token(TokenType::Minus)),
            Some('+') => Ok(self.add_token(TokenType::Plus)),
            Some(';') => Ok(self.add_token(TokenType::Semicolon)),
            Some('*') => Ok(self.add_token(TokenType::Star)),
            Some('!') => {
                if self.match_char('=') {
                    Ok(self.add_token(TokenType::BangEqual))
                } else {
                    Ok(self.add_token(TokenType::Bang))
                }
            }
            Some('=') => {
                if self.match_char('=') {
                    Ok(self.add_token(TokenType::EqualEqual))
                } else {
                    Ok(self.add_token(TokenType::Equal))
                }
            }
            Some('<') => {
                if self.match_char('=') {
                    Ok(self.add_token(TokenType::LessEqual))
                } else {
                    Ok(self.add_token(TokenType::Less))
                }
            }
            Some('>') => {
                if self.match_char('=') {
                    Ok(self.add_token(TokenType::GreaterEqual))
                } else {
                    Ok(self.add_token(TokenType::Greater))
                }
            }
            Some('/') => {
                if self.match_char('/') {
                    self.skip_oneline_comment()
                } else if self.match_char('*') {
                    self.skip_multiline_comment()
                } else {
                    Ok(self.add_token(TokenType::Slash))
                }
            }
            Some(' ') | Some('\r') | Some('\t') => Ok(()),
            Some('\n') => {
                self.line += 1;
                Ok(())
            }
            Some('"') => {
                self.make_string()
            }
            Some(c) if c.is_digit(10) => {
                self.make_number(c)
            }
            Some(c) if c.is_alphabetic() => {
                self.make_identifier(c)
            }
            _ => Err(TokenError::UnexpectedCharacter(c.unwrap_or('\0'), self.line).into()),
        }
    }

    fn make_string(&mut self) -> TokenResult<()> {
        let mut string = String::new();
        while let Some(c) = self.advance() {
            if c == '"' {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            string.push(c);
        }
        if self.is_at_end() {
            return Err(TokenError::UnterminatedString(self.line).into());
        }
        self.add_token(TokenType::String(string));
        Ok(())
    }

    fn make_number(&mut self, first_digit: char) -> TokenResult<()> {
        let mut number = first_digit.to_string();
        while let Some(c) = self.advance() {
            if !c.is_digit(10) && c != '.' {
                break;
            }
            number.push(c);
        }
        match number.parse::<f64>() {
            Ok(num) => {
                self.add_token(TokenType::Number(num));
                Ok(())
            }
            Err(_) => Err(TokenError::InvalidNumber(number, self.line).into()),
        }
    }

    fn make_identifier(&mut self, first_char: char) -> TokenResult<()> {
        let mut identifier = first_char.to_string();
        while let Some(c) = self.advance() {
            if !c.is_alphanumeric() && c != '_' {
                break;
            }
            identifier.push(c);
        }
        let token_type = match identifier.as_str() {
            "var" => TokenType::Var,
            "fun" => TokenType::Fun,
            "for" => TokenType::For,
            "and" => TokenType::And,
            "else" => TokenType::Else,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "while" => TokenType::While,
            _ => TokenType::Identifier(identifier),
        };
        self.add_token(token_type);
        Ok(())
    }

    fn skip_oneline_comment(&mut self) -> TokenResult<()> {
        while let Some(c) = self.advance() {
            if c == '\n' {
                break;
            }
        }
        Ok(())
    }

    fn skip_multiline_comment(&mut self) -> TokenResult<()> {
        while let Some(c) = self.advance() {
            if c == '*' && self.match_char('/') {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
        }
        if self.is_at_end() {
            return Err(TokenError::UnterminatedComment(self.line).into());
        } else {
            Ok(())
        }
    }

    fn add_token(&mut self, kind: TokenType) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(kind, text.to_string(), self.line));
    }
}
