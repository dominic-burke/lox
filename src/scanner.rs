use crate::token::{Token, TokenError, TokenResult, TokenType};

#[derive(Debug, Clone, PartialEq)]
pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    errors: Vec<TokenError>,
    start: usize,
    current: usize,
    line: usize,
}
impl Scanner {
    pub fn new(source: impl Into<String>) -> Self {
        Scanner {
            source: source.into().chars().collect::<Vec<char>>(),
            tokens: Vec::new(),
            errors: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    /// Scans the whole source, returning the tokens that were recognised
    /// alongside any lexical errors. Calling this twice is safe: it rescans
    /// from the start rather than appending to the previous result.
    pub fn scan_tokens(&mut self) -> (Vec<Token>, Vec<TokenError>) {
        self.tokens.clear();
        self.errors.clear();
        self.start = 0;
        self.current = 0;
        self.line = 1;

        while !self.is_at_end() {
            self.start = self.current;
            if let Err(e) = self.scan_token() {
                self.errors.push(e);
            }
        }
        (self.tokens.clone(), self.errors.clone())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> Option<char> {
        match self.source.get(self.current).copied() {
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
        if let Some(c) = self.source.get(self.current) {
            if c != &expected {
                return false;
            }
        } else {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.current).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.current + 1).copied()
    }

    fn scan_token(&mut self) -> TokenResult<()> {
        let c = self.advance();
        match c {
            Some('(') => self.add_token(TokenType::LeftParen),
            Some(')') => self.add_token(TokenType::RightParen),
            Some('{') => self.add_token(TokenType::LeftBrace),
            Some('}') => self.add_token(TokenType::RightBrace),
            Some(',') => self.add_token(TokenType::Comma),
            Some('.') => self.add_token(TokenType::Dot),
            Some('-') => self.add_token(TokenType::Minus),
            Some('+') => self.add_token(TokenType::Plus),
            Some(';') => self.add_token(TokenType::Semicolon),
            Some('*') => self.add_token(TokenType::Star),
            Some('!') => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            Some('=') => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            Some('<') => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            Some('>') => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            Some('/') => {
                if self.match_char('/') {
                    self.skip_oneline_comment()
                } else if self.match_char('*') {
                    self.skip_multiline_comment()
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            Some(' ') | Some('\r') | Some('\t') => Ok(()),
            Some('\n') => {
                self.line += 1;
                Ok(())
            }
            Some('"') => self.make_string(),
            Some(c) if c.is_ascii_digit() => self.make_number(c),
            Some(c) if c.is_ascii_alphabetic() => self.make_identifier(c),
            _ => Err(TokenError::UnexpectedCharacter(
                c.unwrap_or('\0'),
                self.line,
            )),
        }
    }

    fn make_string(&mut self) -> TokenResult<()> {
        let mut string = String::new();
        let mut terminated = false;
        while let Some(c) = self.advance() {
            if c == '"' {
                terminated = true;
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            string.push(c);
        }
        if !terminated {
            return Err(TokenError::UnterminatedString(self.line));
        }
        self.add_token(TokenType::String(string))
    }

    fn make_number(&mut self, first_digit: char) -> TokenResult<()> {
        let mut number = first_digit.to_string();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
                number.push(c);
            } else if c == '.' && self.peek_next().is_some_and(|d| d.is_ascii_digit()) {
                self.advance();
                number.push(c);
                // safe unwrap because we check in self.peek_next
                number.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        match number.parse::<f64>() {
            Ok(num) => self.add_token(TokenType::Number(num)),
            Err(_) => Err(TokenError::InvalidNumber(number, self.line)),
        }
    }

    fn make_identifier(&mut self, first_char: char) -> TokenResult<()> {
        let mut identifier = first_char.to_string();
        while let Some(c) = self.peek() {
            if !c.is_ascii_alphanumeric() && c != '_' {
                break;
            }
            self.advance();
            identifier.push(c);
        }
        let token_type = match identifier.as_str() {
            "var" => TokenType::Var,
            "fun" => TokenType::Fun,
            "for" => TokenType::For,
            "and" => TokenType::And,
            "else" => TokenType::Else,
            "if" => TokenType::If,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "while" => TokenType::While,
            _ => TokenType::Identifier(identifier),
        };
        self.add_token(token_type)
    }

    fn skip_oneline_comment(&mut self) -> TokenResult<()> {
        while let Some(c) = self.advance() {
            if c == '\n' {
                self.line += 1;
                break;
            }
        }
        Ok(())
    }

    fn skip_multiline_comment(&mut self) -> TokenResult<()> {
        while let Some(c) = self.advance() {
            if c == '*' && self.match_char('/') {
                return Ok(());
            } else if c == '\n' {
                self.line += 1;
            }
        }
        Err(TokenError::UnterminatedComment(self.line))
    }

    fn add_token(&mut self, kind: TokenType) -> TokenResult<()> {
        let text = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(kind, text, self.line));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(source: &str) -> Vec<Token> {
        Scanner::new(source.to_string()).scan_tokens().0
    }

    fn token(kind: TokenType, lexeme: &str, line: usize) -> Token {
        Token::new(kind, lexeme.to_string(), line)
    }

    #[test]
    fn scans_single_and_double_character_tokens() {
        let source = "( ) { } , . - + ; * ! != = == < <= > >= /";
        let expected = vec![
            token(TokenType::LeftParen, "(", 1),
            token(TokenType::RightParen, ")", 1),
            token(TokenType::LeftBrace, "{", 1),
            token(TokenType::RightBrace, "}", 1),
            token(TokenType::Comma, ",", 1),
            token(TokenType::Dot, ".", 1),
            token(TokenType::Minus, "-", 1),
            token(TokenType::Plus, "+", 1),
            token(TokenType::Semicolon, ";", 1),
            token(TokenType::Star, "*", 1),
            token(TokenType::Bang, "!", 1),
            token(TokenType::BangEqual, "!=", 1),
            token(TokenType::Equal, "=", 1),
            token(TokenType::EqualEqual, "==", 1),
            token(TokenType::Less, "<", 1),
            token(TokenType::LessEqual, "<=", 1),
            token(TokenType::Greater, ">", 1),
            token(TokenType::GreaterEqual, ">=", 1),
            token(TokenType::Slash, "/", 1),
        ];

        assert_eq!(scan(source), expected);
    }

    #[test]
    fn scans_keywords_and_identifiers() {
        let source = "var fun for and else if nil or print return super this while value2";
        let expected = vec![
            token(TokenType::Var, "var", 1),
            token(TokenType::Fun, "fun", 1),
            token(TokenType::For, "for", 1),
            token(TokenType::And, "and", 1),
            token(TokenType::Else, "else", 1),
            token(TokenType::If, "if", 1),
            token(TokenType::Nil, "nil", 1),
            token(TokenType::Or, "or", 1),
            token(TokenType::Print, "print", 1),
            token(TokenType::Return, "return", 1),
            token(TokenType::Super, "super", 1),
            token(TokenType::This, "this", 1),
            token(TokenType::While, "while", 1),
            token(TokenType::Identifier("value2".to_string()), "value2", 1),
        ];

        assert_eq!(scan(source), expected);
    }

    #[test]
    fn scans_literals_and_tracks_lines() {
        let source = "123.45 \"hello\nworld\" \"\"\nname";
        let expected = vec![
            token(TokenType::Number(123.45), "123.45", 1),
            token(
                TokenType::String("hello\nworld".to_string()),
                "\"hello\nworld\"",
                2,
            ),
            token(TokenType::String(String::new()), "\"\"", 2),
            token(TokenType::Identifier("name".to_string()), "name", 3),
        ];

        assert_eq!(scan(source), expected);
    }

    #[test]
    fn skips_single_and_multiline_comments() {
        let source = "print 1; // ignored\n/* ignored\n * too */ print 2;";
        let expected = vec![
            token(TokenType::Print, "print", 1),
            token(TokenType::Number(1.0), "1", 1),
            token(TokenType::Semicolon, ";", 1),
            token(TokenType::Print, "print", 3),
            token(TokenType::Number(2.0), "2", 3),
            token(TokenType::Semicolon, ";", 3),
        ];

        assert_eq!(scan(source), expected);
    }

    #[test]
    fn reports_unterminated_strings_and_comments() {
        let mut string_scanner = Scanner::new("\"unterminated".to_string());
        assert_eq!(
            string_scanner.scan_token(),
            Err(TokenError::UnterminatedString(1))
        );

        let mut comment_scanner = Scanner::new("/* unterminated".to_string());
        assert_eq!(
            comment_scanner.scan_token(),
            Err(TokenError::UnterminatedComment(1))
        );
    }

    #[test]
    fn test_func_calls_work() {
        let mut string_scanner = Scanner::new("print(\"Hello, World!\");".to_string());
        assert_eq!(
            string_scanner.scan_tokens().0,
            vec![
                token(TokenType::Print, "print", 1),
                token(TokenType::LeftParen, "(", 1),
                token(
                    TokenType::String("Hello, World!".to_string()),
                    "\"Hello, World!\"",
                    1
                ),
                token(TokenType::RightParen, ")", 1),
                token(TokenType::Semicolon, ";", 1),
            ]
        );
    }
    #[test]
    fn test_comment_returns_empty_no_errors() {
        let string = String::from("/* */");
        let mut string_scanner = Scanner::new(string);
        assert_eq!(string_scanner.scan_tokens().0, Vec::new());
        assert_eq!(string_scanner.scan_tokens().1, Vec::new());
    }

    #[test]
    fn test_double_decimal() {
        let string = String::from("123.45 1.2.3");
        let expected_err = vec![TokenError::InvalidNumber(String::from("1.2.3"), 1)];
        let expected_tok = vec![token(TokenType::Number(123.45), "123.45", 1)];
        let mut string_scanner = Scanner::new(string);
        assert_eq!(string_scanner.scan_tokens().0, expected_tok);
        assert_eq!(string_scanner.scan_tokens().1, expected_err);
    }

    #[test]
    fn test_unexpected_char() {
        let string = String::from("chocolate || mint");
        let expected_err = vec![
            TokenError::UnexpectedCharacter('|', 1),
            TokenError::UnexpectedCharacter('|', 1),
        ];
        let expected_tok = vec![
            token(
                TokenType::Identifier(String::from("chocolate")),
                "chocolate",
                1,
            ),
            token(TokenType::Identifier(String::from("mint")), "mint", 1),
        ];
        let mut string_scanner = Scanner::new(string);
        assert_eq!(string_scanner.scan_tokens().0, expected_tok);
        assert_eq!(string_scanner.scan_tokens().1, expected_err);
    }

    #[test]
    fn test_malformed_float() {
        let string = String::from("1.foo");
        let expected_err: Vec<TokenError> = Vec::new();
        let expected_tok = vec![
            token(TokenType::Number(1.0), "1", 1),
            token(TokenType::Dot, ".", 1),
            token(TokenType::Identifier(String::from("foo")), "foo", 1),
        ];
        let mut string_scanner = Scanner::new(string);
        assert_eq!(string_scanner.scan_tokens().0, expected_tok);
        assert_eq!(string_scanner.scan_tokens().1, expected_err);
    }
}
