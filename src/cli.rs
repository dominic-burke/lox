use std::fmt;
use std::io::{self, Write};

use crate::interpreter::Interpreter;
use crate::parser::{ParseError, Parser, Stmt};
use crate::scanner::Scanner;
use crate::token::{Token, TokenError};
use crate::value::{RuntimeError, Value};

/// Exit codes follow the conventions from Crafting Interpreters / sysexits.h.
pub const EX_USAGE: i32 = 64;
pub const EX_DATAERR: i32 = 65;
pub const EX_SOFTWARE: i32 = 70;
pub const EX_IOERR: i32 = 74;

#[derive(Debug)]
pub enum LoxError {
    Scan(Vec<TokenError>),
    Parse(ParseError),
    Runtime(RuntimeError),
    Io(io::Error),
}

impl LoxError {
    pub fn exit_code(&self) -> i32 {
        match self {
            LoxError::Scan(_) | LoxError::Parse(_) => EX_DATAERR,
            LoxError::Runtime(_) => EX_SOFTWARE,
            LoxError::Io(_) => EX_IOERR,
        }
    }
}

impl fmt::Display for LoxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxError::Scan(errors) => {
                for (index, error) in errors.iter().enumerate() {
                    if index > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{error}")?;
                }
                Ok(())
            }
            LoxError::Parse(error) => write!(f, "{error}"),
            LoxError::Runtime(error) => write!(f, "Runtime error: {error}"),
            LoxError::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for LoxError {}

impl From<ParseError> for LoxError {
    fn from(error: ParseError) -> Self {
        LoxError::Parse(error)
    }
}

impl From<RuntimeError> for LoxError {
    fn from(error: RuntimeError) -> Self {
        LoxError::Runtime(error)
    }
}

impl From<io::Error> for LoxError {
    fn from(error: io::Error) -> Self {
        LoxError::Io(error)
    }
}

pub fn run_file(path: &str) -> Result<(), LoxError> {
    let source = std::fs::read_to_string(path)?;
    run(&source)
}

pub fn run(source: &str) -> Result<(), LoxError> {
    let mut interpreter = Interpreter::new();
    run_with(&mut interpreter, source).map(|_| ())
}

/// Scans, parses and executes `source` on an existing interpreter, so that a
/// REPL can keep its globals between lines.
///
/// Returns `Some(value)` when the source is a single expression, which the REPL
/// echoes back to the user.
pub fn run_with(interpreter: &mut Interpreter, source: &str) -> Result<Option<Value>, LoxError> {
    let tokens = scan(source)?;
    let program = Parser::new(&tokens).parse_program()?;
    interpret_program(interpreter, program)
}

/// Like [`run_with`], but a line that is just an expression needs no trailing
/// semicolon, so the REPL doubles as a calculator.
pub fn run_line(interpreter: &mut Interpreter, source: &str) -> Result<Option<Value>, LoxError> {
    let tokens = scan(source)?;

    match Parser::new(&tokens).parse_program() {
        Ok(program) => interpret_program(interpreter, program),
        // Not a valid statement: maybe the line is a bare expression.
        Err(program_error) => match Parser::new(&tokens).parse_expression_only() {
            Ok(expr) => Ok(Some(interpreter.evaluate_expression(&expr)?)),
            Err(_) => Err(program_error.into()),
        },
    }
}

fn interpret_program(
    interpreter: &mut Interpreter,
    program: Vec<Stmt>,
) -> Result<Option<Value>, LoxError> {
    // A lone expression statement echoes its value; everything else runs
    // normally and prints only through `print`.
    if let [Stmt::Expression(expr)] = program.as_slice() {
        return Ok(Some(interpreter.evaluate_expression(expr)?));
    }

    interpreter.interpret(&program)?;
    Ok(None)
}

fn scan(source: &str) -> Result<Vec<Token>, LoxError> {
    let mut scanner = Scanner::new(source);
    let (tokens, errors) = scanner.scan_tokens();
    if !errors.is_empty() {
        return Err(LoxError::Scan(errors));
    }
    Ok(tokens)
}

pub fn run_prompt() -> Result<(), LoxError> {
    let mut interpreter = Interpreter::new();
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        line.clear();
        if stdin.read_line(&mut line)? == 0 {
            println!();
            break;
        }
        if line.trim().is_empty() {
            continue;
        }

        // Errors don't end the session; they just get reported. Each line is
        // parsed on its own, so multi-line constructs aren't supported here.
        match run_line(&mut interpreter, &line) {
            Ok(Some(value)) => println!("{value}"),
            Ok(None) => {}
            Err(error) => eprintln!("{error}"),
        }
    }
    Ok(())
}
