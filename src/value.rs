use std::fmt;
use std::rc::Rc;

use crate::interpreter::Interpreter;
use crate::token::Token;

/// Anything that can be called with the `(...)` syntax: native functions and
/// user-defined Lox functions.
pub trait LoxCallable {
    fn arity(&self) -> usize;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;

    fn name(&self) -> String;

    /// How the callable prints from Lox, e.g. `<fn foo>` or `<native fn>`.
    fn display(&self) -> String {
        format!("<fn {}>", self.name())
    }
}

/// A Lox runtime value. Values are cheap to clone: strings are the only heap
/// data and callables are reference counted.
#[derive(Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Callable(Rc<dyn LoxCallable>),
}

impl Value {
    /// `nil` and `false` are falsey; everything else (including `0` and `""`)
    /// is truthy.
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Nil | Value::Bool(false))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write_number(f, *n),
            Value::String(s) => write!(f, "{s}"),
            Value::Callable(callable) => write!(f, "{}", callable.display()),
        }
    }
}

fn write_number(f: &mut fmt::Formatter<'_>, n: f64) -> fmt::Result {
    if n.is_nan() {
        write!(f, "NaN")
    } else if n.is_infinite() {
        if n.is_sign_positive() {
            write!(f, "Infinity")
        } else {
            write!(f, "-Infinity")
        }
    } else if n == 0.0 && n.is_sign_negative() {
        // Reached by `-0`; keep the sign, as jlox does.
        write!(f, "-0")
    } else if n.fract() == 0.0 && n.abs() < 1e15 {
        // Lox prints whole numbers without a trailing ".0".
        write!(f, "{}", n as i64)
    } else {
        write!(f, "{n}")
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            // Note: NaN == NaN is false, which matches IEEE semantics.
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Callable(a), Value::Callable(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// A Lox runtime error, optionally tagged with the source line it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    message: String,
    line: Option<usize>,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        RuntimeError {
            message: message.into(),
            line: None,
        }
    }

    pub fn at(token: &Token, message: impl Into<String>) -> Self {
        RuntimeError {
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

    /// Attaches a line number if the error doesn't already carry one. Used to
    /// pin errors raised deep inside a native function to the call site.
    pub fn with_line(self, line: usize) -> Self {
        if self.line.is_some() {
            self
        } else {
            RuntimeError {
                message: self.message,
                line: Some(line),
            }
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(line) = self.line {
            write!(f, "\n[line {line}]")?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {}
