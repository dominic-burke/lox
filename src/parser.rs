use crate::token::{Token, TokenType};

type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone, PartialEq)]
enum ParseError {
    InvalidToken,
    UnexpectedTokenEnd,
    ExpectedButNotFound(TokenType),
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ParseError::ExpectedButNotFound(tt) => write!(f, "Wanted to find {tt} but couldn't"),
            ParseError::InvalidToken => write!(f, "Invalid token found"),
            ParseError::UnexpectedTokenEnd => write!(f, "Tokens came to an end unexpectedly")
        }
        
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    fn parse(&mut self) -> ParseResult<Expression> {
        self.expression()
    }
    fn expression(&mut self) -> ParseResult<Expression> {
        self.equality()
    }

    fn equality(&mut self) -> ParseResult<Expression> {
        let mut expr = self.comparison()?;

        while let Some(t) = self.match_any(&[TokenType::Bang, TokenType::BangEqual]) {
            let op = t;
            let right = self.comparison()?;
            expr = Expression::BinaryExpr {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.term()?;

        while let Some(t) = self.match_any(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = t;
            let right = self.term()?;
            expr = Expression::BinaryExpr {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expression> {
        let mut expr = self.factor()?;

        while let Some(t) = self.match_any(&[TokenType::Minus, TokenType::Plus]) {
            let op = t;
            let right = self.factor()?;
            expr = Expression::BinaryExpr {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expression> {
        let mut expr = self.unary()?;

        while let Some(t) = self.match_any(&[TokenType::Slash, TokenType::Star]) {
            let op = t;
            let right = self.unary()?;
            expr = Expression::BinaryExpr {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expression> {
        if let Some(t) = self.match_any(&[TokenType::Bang, TokenType::Minus]) {
            let op = t;
            let right = self.unary()?;
            return Ok(Expression::Unary {
                operator: op,
                right: Box::new(right),
            });
        }
        return Ok(self.primary()?);
    }

    fn primary(&mut self) -> ParseResult<Expression> {
        if let Some(_) = self.match_any(&[TokenType::True]) {
            return Ok(Expression::Literal {
                value: Value::Bool(true),
            });
        }
        if let Some(_) = self.match_any(&[TokenType::False]) {
            return Ok(Expression::Literal {
                value: Value::Bool(false),
            });
        }
        if let Some(_) = self.match_any(&[TokenType::Nil]) {
            return Ok(Expression::Literal { value: Value::Nil });
        }

        if let Some(t) = self.match_any(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen)?;
            return Ok(Expression::Grouping {
                expr: Box::new(expr),
            });
        }

        if let Some(t) = self.peek() {
            return match t.get_kind() {
                TokenType::Number(num) => Ok(Expression::Literal {
                    value: Value::Number(*num),
                }),
                TokenType::String(str) => Ok(Expression::Literal {
                    value: Value::String(str.clone()),
                }),
                _ => Err(ParseError::InvalidToken),
            };
        }
        Err(ParseError::UnexpectedTokenEnd)
    }

    // just have to find this token and keep the parser going
    fn consume(&mut self, tt: &TokenType) -> ParseResult<()> {
        if self.peek().is_some_and(|x| x.get_kind() == tt) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::ExpectedButNotFound(tt.clone()))
        }
    }

    fn match_any(&mut self, kinds: &[TokenType]) -> Option<Token> {
        if kinds.iter().any(|k| self.check(k)) {
            self.advance()
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.current).cloned()?;
        self.current += 1;
        Some(token)
    }

    fn check(&self, tt: &TokenType) -> bool {
        !self.is_at_end() && self.peek().is_some_and(|c| *c.get_kind() == *tt)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&'a Token> {
        self.current.checked_sub(1).and_then(|i| self.tokens.get(i))
    }
}

enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    Nil,
}
enum Expression {
    BinaryExpr {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Literal {
        value: Value,
    },
    Unary {
        operator: Token,
        right: Box<Expression>,
    },
    Grouping {
        expr: Box<Expression>,
    },
}

impl Expression {
    fn evaluate(&self) -> Value {
        match self {
            Expression::BinaryExpr {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate();
                let right = right.evaluate();

                todo!()
            }
            _ => todo!(),
        }
    }
}
