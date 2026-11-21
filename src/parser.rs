use std::fmt;
use std::rc::Rc;

use crate::token::{Token, TokenType};
use crate::value::Value;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    message: String,
    line: Option<usize>,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        ParseError {
            message: message.into(),
            line: None,
        }
    }

    pub fn at(token: &Token, message: impl Into<String>) -> Self {
        ParseError {
            message: message.into(),
            line: Some(token.line()),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn line(&self) -> Option<usize> {
        self.line
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(line) = self.line {
            write!(f, "\n[line {line}]")?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// A function declaration. Kept behind an `Rc` so creating a closure at
/// runtime doesn't have to copy the body.
#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expression),
    Print(Expression),
    Var {
        name: Token,
        initializer: Option<Expression>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expression,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expression,
        body: Box<Stmt>,
    },
    Function(Rc<FunctionDecl>),
    Return {
        keyword: Token,
        value: Option<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
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
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expression>,
    },
    Logical {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        paren: Token,
        arguments: Vec<Expression>,
    },
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
    /// Non-zero while parsing a function body; top-level `return` is an error.
    function_depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            current: 0,
            function_depth: 0,
        }
    }

    pub fn parse_program(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    /// Parses one expression that must consume the entire token stream. The
    /// REPL uses this so a line like `1 + 2` evaluates without needing a
    /// trailing semicolon.
    pub fn parse_expression_only(&mut self) -> ParseResult<Expression> {
        let expr = self.expression()?;
        if let Some(token) = self.peek() {
            return Err(ParseError::at(token, "Expect expression."));
        }
        Ok(expr)
    }

    // declaration → funDecl | varDecl | statement
    fn declaration(&mut self) -> ParseResult<Stmt> {
        if self.match_any(&[TokenType::Var]).is_some() {
            return self.var_declaration();
        }
        if self.match_any(&[TokenType::Fun]).is_some() {
            return self.function();
        }
        self.statement()
    }

    // `var` has already been consumed.
    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume_identifier("Expect variable name.")?;
        let initializer = if self.match_any(&[TokenType::Equal]).is_some() {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    // `fun` has already been consumed.
    fn function(&mut self) -> ParseResult<Stmt> {
        let name = self.consume_identifier("Expect function name.")?;
        self.consume(&TokenType::LeftParen, "Expect '(' after function name.")?;
        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(ParseError::at(
                        self.peek().unwrap_or(&name),
                        "Can't have more than 255 parameters.",
                    ));
                }
                params.push(self.consume_identifier("Expect parameter name.")?);
                if self.match_any(&[TokenType::Comma]).is_none() {
                    break;
                }
            }
        }
        self.consume(&TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(&TokenType::LeftBrace, "Expect '{' before function body.")?;

        self.function_depth += 1;
        let body = self.block();
        self.function_depth -= 1;

        Ok(Stmt::Function(Rc::new(FunctionDecl {
            name,
            params,
            body: body?,
        })))
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_any(&[TokenType::For]).is_some() {
            return self.for_statement();
        }
        if self.match_any(&[TokenType::If]).is_some() {
            return self.if_statement();
        }
        if self.match_any(&[TokenType::Print]).is_some() {
            return self.print_statement();
        }
        if self.match_any(&[TokenType::Return]).is_some() {
            return self.return_statement();
        }
        if self.match_any(&[TokenType::While]).is_some() {
            return self.while_statement();
        }
        if self.match_any(&[TokenType::LeftBrace]).is_some() {
            return Ok(Stmt::Block(self.block()?));
        }
        self.expression_statement()
    }

    fn block(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_any(&[TokenType::Else]).is_some() {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after condition.")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
    }

    /// `for` is sugar: it desugars into a block containing an initializer and a
    /// `while` loop, which keeps the interpreter free of a fourth loop form.
    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_any(&[TokenType::Semicolon]).is_some() {
            None
        } else if self.match_any(&[TokenType::Var]).is_some() {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.check(&TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        let condition = condition.unwrap_or(Expression::Literal {
            value: Value::Bool(true),
        });
        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let keyword = self
            .previous()
            .cloned()
            .ok_or_else(|| ParseError::new("Expect 'return'."))?;

        let value = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&TokenType::Semicolon, "Expect ';' after return value.")?;

        if self.function_depth == 0 {
            return Err(ParseError::at(
                &keyword,
                "Can't return from top-level code.",
            ));
        }
        Ok(Stmt::Return { keyword, value })
    }

    // expression → assignment
    fn expression(&mut self) -> ParseResult<Expression> {
        self.assignment()
    }

    // assignment → IDENTIFIER "=" assignment | logic_or
    //
    // The LHS is only an assignment target if it is a bare identifier followed
    // by `=`, which two tokens of lookahead is enough to decide.
    fn assignment(&mut self) -> ParseResult<Expression> {
        let starts_with_identifier = self
            .peek()
            .is_some_and(|t| matches!(t.get_kind(), TokenType::Identifier(_)));

        if starts_with_identifier && self.check_next(&TokenType::Equal) {
            let name = self
                .advance()
                .ok_or_else(|| ParseError::new("Expect variable name."))?;
            self.advance(); // the '='
            let value = self.assignment()?;
            return Ok(Expression::Assign {
                name,
                value: Box::new(value),
            });
        }

        let expr = self.logic_or()?;
        // Anything else followed by `=` can't be assigned to, e.g. `(a) = 1`.
        if self.check(&TokenType::Equal) {
            let token = self
                .peek()
                .ok_or_else(|| ParseError::new("Invalid assignment target."))?;
            return Err(ParseError::at(token, "Invalid assignment target."));
        }
        Ok(expr)
    }

    // logic_or → logic_and ( "or" logic_and )*
    fn logic_or(&mut self) -> ParseResult<Expression> {
        let mut expr = self.logic_and()?;
        while let Some(operator) = self.match_any(&[TokenType::Or]) {
            let right = self.logic_and()?;
            expr = Expression::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // logic_and → equality ( "and" equality )*
    fn logic_and(&mut self) -> ParseResult<Expression> {
        let mut expr = self.equality()?;
        while let Some(operator) = self.match_any(&[TokenType::And]) {
            let right = self.equality()?;
            expr = Expression::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expression> {
        self.binary(
            &[TokenType::EqualEqual, TokenType::BangEqual],
            Self::comparison,
        )
    }

    fn comparison(&mut self) -> ParseResult<Expression> {
        self.binary(
            &[
                TokenType::Greater,
                TokenType::GreaterEqual,
                TokenType::Less,
                TokenType::LessEqual,
            ],
            Self::term,
        )
    }

    fn term(&mut self) -> ParseResult<Expression> {
        self.binary(&[TokenType::Minus, TokenType::Plus], Self::factor)
    }

    fn factor(&mut self) -> ParseResult<Expression> {
        self.binary(&[TokenType::Slash, TokenType::Star], Self::unary)
    }

    /// Parses a left-associative chain of binary operators of one precedence
    /// level, calling `next_fn` for each operand.
    fn binary(
        &mut self,
        ops: &[TokenType],
        next_fn: fn(&mut Self) -> ParseResult<Expression>,
    ) -> ParseResult<Expression> {
        let mut expr = next_fn(self)?;

        while let Some(operator) = self.match_any(ops) {
            let right = next_fn(self)?;
            expr = Expression::BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // unary → ( "!" | "-" | "+" ) unary | call
    fn unary(&mut self) -> ParseResult<Expression> {
        if let Some(operator) =
            self.match_any(&[TokenType::Bang, TokenType::Minus, TokenType::Plus])
        {
            let right = self.unary()?;
            return Ok(Expression::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.call()
    }

    // call → primary ( "(" arguments? ")" )*
    fn call(&mut self) -> ParseResult<Expression> {
        let mut expr = self.primary()?;
        while self.check(&TokenType::LeftParen) {
            expr = self.finish_call(expr)?;
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> ParseResult<Expression> {
        let paren = self
            .advance()
            .ok_or_else(|| ParseError::new("Expect ')' after arguments."))?;

        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(ParseError::at(
                        self.peek().unwrap_or(&paren),
                        "Can't have more than 255 arguments.",
                    ));
                }
                arguments.push(self.expression()?);
                if self.match_any(&[TokenType::Comma]).is_none() {
                    break;
                }
            }
        }
        self.consume(&TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expression::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn primary(&mut self) -> ParseResult<Expression> {
        let Some(token) = self.peek().cloned() else {
            return Err(match self.previous() {
                Some(last) => ParseError::at(last, "Expect expression."),
                None => ParseError::new("Expect expression."),
            });
        };

        let literal = match token.get_kind() {
            TokenType::False => Some(Value::Bool(false)),
            TokenType::True => Some(Value::Bool(true)),
            TokenType::Nil => Some(Value::Nil),
            TokenType::Number(n) => Some(Value::Number(*n)),
            TokenType::String(s) => Some(Value::String(s.clone())),
            _ => None,
        };
        if let Some(value) = literal {
            self.advance();
            return Ok(Expression::Literal { value });
        }

        if matches!(token.get_kind(), TokenType::Identifier(_)) {
            self.advance();
            return Ok(Expression::Variable { name: token });
        }

        if matches!(token.get_kind(), TokenType::LeftParen) {
            self.advance();
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expression::Grouping {
                expr: Box::new(expr),
            });
        }

        Err(ParseError::at(&token, "Expect expression."))
    }

    /// Consumes a token of the given kind and returns it, with a line number
    /// attached to the error when it isn't there.
    fn consume(&mut self, tt: &TokenType, message: &str) -> ParseResult<Token> {
        match self.peek() {
            Some(token) if token.get_kind() == tt => {
                self.advance().ok_or_else(|| ParseError::new(message))
            }
            Some(token) => Err(ParseError::at(token, message)),
            None => Err(match self.previous() {
                Some(last) => ParseError::at(last, message),
                None => ParseError::new(message),
            }),
        }
    }

    fn consume_identifier(&mut self, message: &str) -> ParseResult<Token> {
        match self.peek() {
            Some(token) if matches!(token.get_kind(), TokenType::Identifier(_)) => {
                self.advance().ok_or_else(|| ParseError::new(message))
            }
            Some(token) => Err(ParseError::at(token, message)),
            None => Err(match self.previous() {
                Some(last) => ParseError::at(last, message),
                None => ParseError::new(message),
            }),
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

    fn check_next(&self, tt: &TokenType) -> bool {
        self.tokens
            .get(self.current + 1)
            .is_some_and(|c| *c.get_kind() == *tt)
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
