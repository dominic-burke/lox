use std::io::{self, Write};
use std::rc::Rc;

use crate::environment::{EnvRef, Environment};
use crate::parser::{Expression, FunctionDecl, Stmt};
use crate::token::{Token, TokenType};
use crate::value::{LoxCallable, RuntimeError, Value};

/// How a statement finished: normally, or with a `return` unwinding towards the
/// enclosing function call.
#[derive(Debug, PartialEq)]
pub enum ExecOutcome {
    Normal,
    Return(Value),
}

pub struct Interpreter {
    environment: EnvRef,
    out: Box<dyn Write>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::with_writer(Box::new(io::stdout()))
    }

    /// Builds an interpreter that writes `print` output to `out` (used by the
    /// tests to capture output).
    pub fn with_writer(out: Box<dyn Write>) -> Self {
        let globals = Environment::new_ref();
        globals
            .borrow_mut()
            .define("clock", Value::Callable(Rc::new(Clock)));

        Interpreter {
            environment: globals,
            out,
        }
    }

    pub fn interpret(&mut self, program: &[Stmt]) -> Result<(), RuntimeError> {
        for statement in program {
            match self.execute(statement)? {
                ExecOutcome::Normal => {}
                ExecOutcome::Return(_) => {
                    // The parser already rejects this; belt and braces.
                    return Err(RuntimeError::new("Can't return from top-level code."));
                }
            }
        }
        Ok(())
    }

    /// Evaluates a single expression without executing it as a statement. Used
    /// by the REPL to echo values.
    pub fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        self.evaluate(expr)
    }

    fn execute(&mut self, statement: &Stmt) -> Result<ExecOutcome, RuntimeError> {
        match statement {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(ExecOutcome::Normal)
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                writeln!(self.out, "{value}").map_err(write_error)?;
                Ok(ExecOutcome::Normal)
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(expr)?,
                    None => Value::Nil,
                };
                self.environment.borrow_mut().define(name.lexeme(), value);
                Ok(ExecOutcome::Normal)
            }
            Stmt::Block(statements) => {
                let environment = Environment::new_child(Rc::clone(&self.environment));
                self.execute_block(statements, environment)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)
                } else {
                    Ok(ExecOutcome::Normal)
                }
            }
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    match self.execute(body)? {
                        ExecOutcome::Normal => {}
                        early_exit => return Ok(early_exit),
                    }
                }
                Ok(ExecOutcome::Normal)
            }
            Stmt::Function(declaration) => {
                let function = LoxFunction {
                    declaration: Rc::clone(declaration),
                    closure: Rc::clone(&self.environment),
                };
                self.environment.borrow_mut().define(
                    declaration.name.lexeme(),
                    Value::Callable(Rc::new(function)),
                );
                Ok(ExecOutcome::Normal)
            }
            Stmt::Return { value, .. } => {
                let value = match value {
                    Some(expr) => self.evaluate(expr)?,
                    None => Value::Nil,
                };
                Ok(ExecOutcome::Return(value))
            }
        }
    }

    /// Runs `statements` in `environment`, restoring the previous scope
    /// afterwards even if a statement fails.
    fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: EnvRef,
    ) -> Result<ExecOutcome, RuntimeError> {
        let previous = std::mem::replace(&mut self.environment, environment);
        let mut outcome = Ok(ExecOutcome::Normal);

        for statement in statements {
            match self.execute(statement) {
                Ok(ExecOutcome::Normal) => continue,
                other => {
                    outcome = other;
                    break;
                }
            }
        }

        self.environment = previous;
        outcome
    }

    fn evaluate(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Literal { value } => Ok(value.clone()),
            Expression::Grouping { expr } => self.evaluate(expr),
            Expression::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                self.unary_op(operator, right)
            }
            Expression::BinaryExpr {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                self.binary_op(left, operator, right)
            }
            Expression::Logical {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left)?;
                // Short-circuit: `or` returns the left operand when truthy,
                // `and` returns it when falsey, without evaluating the right.
                let evaluate_right = match operator.get_kind() {
                    TokenType::Or => !left_value.is_truthy(),
                    _ => left_value.is_truthy(),
                };
                if evaluate_right {
                    self.evaluate(right)
                } else {
                    Ok(left_value)
                }
            }
            Expression::Variable { name } => self
                .environment
                .borrow()
                .get(name.lexeme())
                .ok_or_else(|| undefined_variable(name)),
            Expression::Assign { name, value } => {
                let value = self.evaluate(value)?;
                if self
                    .environment
                    .borrow_mut()
                    .assign(name.lexeme(), value.clone())
                {
                    Ok(value)
                } else {
                    Err(undefined_variable(name))
                }
            }
            Expression::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;
                let mut evaluated = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    evaluated.push(self.evaluate(argument)?);
                }
                self.call_value(callee, evaluated, paren)
            }
        }
    }

    fn unary_op(&self, operator: &Token, right: Value) -> Result<Value, RuntimeError> {
        match operator.get_kind() {
            TokenType::Bang => Ok(Value::Bool(!right.is_truthy())),
            // Unary `+` is a no-op on numbers; it exists to make `+x` explicit.
            TokenType::Plus => match right {
                Value::Number(n) => Ok(Value::Number(n)),
                _ => Err(RuntimeError::at(operator, "Operand must be a number.")),
            },
            TokenType::Minus => match right {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(RuntimeError::at(operator, "Operand must be a number.")),
            },
            _ => Err(RuntimeError::at(operator, "Invalid unary operator.")),
        }
    }

    fn binary_op(
        &self,
        left: Value,
        operator: &Token,
        right: Value,
    ) -> Result<Value, RuntimeError> {
        match operator.get_kind() {
            TokenType::EqualEqual => Ok(Value::Bool(left == right)),
            TokenType::BangEqual => Ok(Value::Bool(left != right)),
            TokenType::Plus => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
                _ => Err(RuntimeError::at(
                    operator,
                    "Operands must be two numbers or two strings.",
                )),
            },
            _ => {
                // Everything else is numeric.
                let (a, b) = match (left, right) {
                    (Value::Number(a), Value::Number(b)) => (a, b),
                    _ => return Err(RuntimeError::at(operator, "Operands must be numbers.")),
                };
                let result = match operator.get_kind() {
                    TokenType::Minus => Value::Number(a - b),
                    TokenType::Star => Value::Number(a * b),
                    TokenType::Slash => Value::Number(a / b),
                    TokenType::Greater => Value::Bool(a > b),
                    TokenType::GreaterEqual => Value::Bool(a >= b),
                    TokenType::Less => Value::Bool(a < b),
                    TokenType::LessEqual => Value::Bool(a <= b),
                    _ => return Err(RuntimeError::at(operator, "Invalid binary operator.")),
                };
                Ok(result)
            }
        }
    }

    fn call_value(
        &mut self,
        callee: Value,
        arguments: Vec<Value>,
        paren: &Token,
    ) -> Result<Value, RuntimeError> {
        let Value::Callable(function) = callee else {
            return Err(RuntimeError::at(
                paren,
                "Can only call functions and classes.",
            ));
        };

        if arguments.len() != function.arity() {
            return Err(RuntimeError::at(
                paren,
                format!(
                    "Expected {} arguments but got {}.",
                    function.arity(),
                    arguments.len()
                ),
            ));
        }

        function
            .call(self, arguments)
            .map_err(|error| error.with_line(paren.line()))
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

fn undefined_variable(name: &Token) -> RuntimeError {
    RuntimeError::at(name, format!("Undefined variable '{}'.", name.lexeme()))
}

fn write_error(error: io::Error) -> RuntimeError {
    RuntimeError::new(format!("Could not write output: {error}"))
}

struct LoxFunction {
    declaration: Rc<FunctionDecl>,
    closure: EnvRef,
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let environment = Environment::new_child(Rc::clone(&self.closure));
        {
            let mut scope = environment.borrow_mut();
            for (param, argument) in self.declaration.params.iter().zip(arguments) {
                scope.define(param.lexeme(), argument);
            }
        }

        match interpreter.execute_block(&self.declaration.body, environment)? {
            ExecOutcome::Return(value) => Ok(value),
            ExecOutcome::Normal => Ok(Value::Nil),
        }
    }

    fn name(&self) -> String {
        self.declaration.name.lexeme().to_string()
    }
}

/// Native `clock()`: seconds since the Unix epoch as a Lox number.
struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs_f64())
            .unwrap_or(0.0);
        Ok(Value::Number(seconds))
    }

    fn name(&self) -> String {
        "clock".to_string()
    }

    fn display(&self) -> String {
        "<native fn>".to_string()
    }
}
