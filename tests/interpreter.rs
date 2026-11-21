use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use lox::cli::{run_line, run_with};
use lox::interpreter::Interpreter;

/// Collects everything the interpreter `print`s so assertions can look at it.
#[derive(Clone, Default)]
struct Capture(Rc<RefCell<Vec<u8>>>);

impl Capture {
    fn new() -> Self {
        Capture::default()
    }

    fn contents(&self) -> String {
        String::from_utf8(self.0.borrow().clone()).expect("output is valid UTF-8")
    }
}

impl Write for Capture {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Runs a whole program and returns whatever it printed, or the error message.
fn run(source: &str) -> Result<String, String> {
    let capture = Capture::new();
    let mut interpreter = Interpreter::with_writer(Box::new(capture.clone()));
    run_with(&mut interpreter, source).map_err(|error| error.to_string())?;
    Ok(capture.contents())
}

#[track_caller]
fn assert_output(source: &str, expected: &str) {
    match run(source) {
        Ok(output) => assert_eq!(output, expected, "unexpected output for: {source}"),
        Err(error) => panic!("error running:\n{source}\n{error}"),
    }
}

#[track_caller]
fn assert_error(source: &str, expected_fragment: &str) {
    match run(source) {
        Ok(output) => panic!("expected an error for:\n{source}\ngot output: {output:?}"),
        Err(error) => assert!(
            error.contains(expected_fragment),
            "expected error containing {expected_fragment:?} for:\n{source}\ngot: {error}"
        ),
    }
}

#[test]
fn arithmetic_respects_precedence() {
    assert_output("print 1 + 2 * 3;", "7\n");
    assert_output("print (1 + 2) * 3;", "9\n");
    assert_output("print 7 - 2 - 3;", "2\n");
    assert_output("print 8 / 4 / 2;", "1\n");
    assert_output("print 7 / 2;", "3.5\n");
    assert_output("print -3 + +3;", "0\n");
    assert_output("print !!true;", "true\n");
}

#[test]
fn numbers_print_without_trailing_zero() {
    assert_output("print 1.0;", "1\n");
    assert_output("print 123.45;", "123.45\n");
    assert_output("print -0;", "-0\n");
    assert_output("print 1 / 0;", "Infinity\n");
    assert_output("print -1 / 0;", "-Infinity\n");
    assert_output("print 0 / 0;", "NaN\n");
}

#[test]
fn functions_print_with_their_names() {
    assert_output(
        "fun foo() {}
         print foo;
         print clock;",
        "<fn foo>\n<native fn>\n",
    );
}

#[test]
fn comparisons_and_equality_evaluate_to_bools() {
    assert_output("print 1 < 2;", "true\n");
    assert_output("print 2 <= 2;", "true\n");
    assert_output("print 3 > 4;", "false\n");
    assert_output("print 1 == 1.0;", "true\n");
    assert_output("print \"hi\" == \"hi\";", "true\n");
    assert_output("print \"hi\" != \"ho\";", "true\n");
    assert_output("print nil == false;", "false\n");
    assert_output("print nil == nil;", "true\n");
    assert_output("print 1 < 2 == true;", "true\n");
}

#[test]
fn strings_concatenate() {
    assert_output("print \"a\" + \"b\" + \"c\";", "abc\n");
}

#[test]
fn nil_and_false_are_the_only_falsey_values() {
    assert_output(
        "print !nil; print !false; print !0; print !\"\"; print !clock;",
        "true\ntrue\nfalse\nfalse\nfalse\n",
    );
}

#[test]
fn logical_operators_short_circuit() {
    assert_output(
        "fun boom() { print \"boom\"; return true; }
         print false and boom();
         print true or boom();
         print nil or \"fallback\";
         print \"first\" and \"second\";",
        "false\ntrue\nfallback\nsecond\n",
    );
}

#[test]
fn variables_are_assigned_and_shadowed() {
    assert_output(
        "var a = \"global\";
         { var a = \"block\"; print a; }
         print a;",
        "block\nglobal\n",
    );
}

#[test]
fn assignment_mutates_the_existing_binding() {
    assert_output("var a = 1; a = a + 2; print a;", "3\n");
    assert_output(
        "var a = \"outer\";
         { a = \"inner\"; }
         print a;",
        "inner\n",
    );
}

#[test]
fn uninitialised_variables_are_nil() {
    assert_output("var a; print a;", "nil\n");
}

#[test]
fn if_else_picks_a_branch() {
    assert_output(
        "if (1 > 2) print \"then\"; else print \"else\";
         if (nil) print \"then\"; else print \"else\";
         if (\"truthy\") print \"then\";",
        "else\nelse\nthen\n",
    );
}

#[test]
fn while_loops_until_the_condition_is_false() {
    assert_output(
        "var i = 0;
         while (i < 3) { print i; i = i + 1; }",
        "0\n1\n2\n",
    );
}

#[test]
fn for_loops_are_desugared_and_scoped() {
    assert_output("for (var i = 0; i < 3; i = i + 1) print i;", "0\n1\n2\n");
    assert_error(
        "for (var i = 0; i < 3; i = i + 1) {} print i;",
        "Undefined variable 'i'.",
    );
    assert_output(
        "var j = 0; for (; j < 2;) { print j; j = j + 1; }",
        "0\n1\n",
    );
}

#[test]
fn functions_return_values_and_default_to_nil() {
    assert_output(
        "fun add(a, b) { return a + b; }
         fun nothing() {}
         print add(1, 2);
         print nothing();",
        "3\nnil\n",
    );
}

#[test]
fn recursion_works() {
    assert_output(
        "fun fib(n) { if (n < 2) return n; return fib(n - 1) + fib(n - 2); }
         print fib(10);",
        "55\n",
    );
}

#[test]
fn closures_capture_their_environment() {
    assert_output(
        "fun makeCounter() {
           var i = 0;
           fun count() { i = i + 1; print i; }
           return count;
         }
         var counter = makeCounter();
         counter();
         counter();
         var other = makeCounter();
         other();",
        "1\n2\n1\n",
    );
}

#[test]
fn functions_are_first_class_values() {
    assert_output(
        "fun twice(f, x) { return f(f(x)); }
         fun inc(n) { return n + 1; }
         print twice(inc, 5);
         print inc;",
        "7\n<fn inc>\n",
    );
}

#[test]
fn return_unwinds_through_blocks_and_loops() {
    assert_output(
        "fun find(limit) {
           var i = 0;
           while (true) {
             if (i == limit) { return i; }
             i = i + 1;
           }
         }
         print find(4);",
        "4\n",
    );
}

#[test]
fn clock_is_a_native_function() {
    match run("print clock() > 1600000000;") {
        Ok(output) => assert_eq!(output, "true\n"),
        Err(error) => panic!("clock() failed: {error}"),
    }
    assert_error("clock(1);", "Expected 0 arguments but got 1.");
}

#[test]
fn undefined_variables_are_runtime_errors() {
    assert_error("print missing;", "Undefined variable 'missing'.");
    assert_error("missing = 1;", "Undefined variable 'missing'.");
    // Line numbers are reported too.
    assert_error("var a = 1;\nprint a;\nprint b;", "[line 3]");
}

#[test]
fn bad_operands_are_runtime_errors() {
    assert_error("print \"a\" - 1;", "Operands must be numbers.");
    assert_error("print -\"a\";", "Operand must be a number.");
    assert_error(
        "print 1 + \"a\";",
        "Operands must be two numbers or two strings.",
    );
}

#[test]
fn calling_a_non_callable_is_an_error() {
    assert_error("var a = 1; a();", "Can only call functions and classes.");
}

#[test]
fn wrong_arity_is_an_error() {
    assert_error(
        "fun f(a, b) { return a; } f(1);",
        "Expected 2 arguments but got 1.",
    );
}

#[test]
fn parse_errors_are_reported_with_lines() {
    assert_error("print 1", "Expect ';' after value.");
    assert_error("print 1 + ;", "Expect expression.");
    assert_error("var = 1;", "Expect variable name.");
    assert_error("if (true) { print 1;", "Expect '}' after block.");
    assert_error("print 1;\nprint 2 +;", "[line 2]");
}

#[test]
fn invalid_assignment_targets_are_parse_errors() {
    assert_error("(a) = 1;", "Invalid assignment target.");
    assert_error("a + b = 1;", "Invalid assignment target.");
}

#[test]
fn top_level_return_is_a_parse_error() {
    assert_error("return 1;", "Can't return from top-level code.");
    // ...but returning from inside a function body is fine.
    assert_output("fun f() { return 1; } print f();", "1\n");
}

#[test]
fn scanner_errors_stop_execution() {
    assert_error("print \"unterminated;", "Unterminated string.");
    assert_error("print 1 @ 2;", "Unexpected character '@'.");
}

#[test]
fn a_single_expression_is_returned_for_the_repl() {
    let capture = Capture::new();
    let mut interpreter = Interpreter::with_writer(Box::new(capture.clone()));

    // In the REPL a bare expression needs no semicolon.
    let value = run_line(&mut interpreter, "1 + 2 * 3").expect("expression should evaluate");
    assert_eq!(value.map(|v| v.to_string()), Some("7".to_string()));

    // With a semicolon it's an expression statement, which also echoes.
    let value = run_line(&mut interpreter, "1 + 1;").expect("expression should evaluate");
    assert_eq!(value.map(|v| v.to_string()), Some("2".to_string()));

    // Statements return nothing, and `print` still goes to the writer.
    let value = run_line(&mut interpreter, "print 1 + 1;").expect("statement should run");
    assert_eq!(value, None);
    assert_eq!(capture.contents(), "2\n");
}

#[test]
fn files_require_semicolons() {
    // The lenient bare-expression path is REPL-only; a script stays strict.
    assert_error("1 + 2", "Expect ';' after expression.");
}

#[test]
fn the_repl_keeps_globals_between_lines() {
    let capture = Capture::new();
    let mut interpreter = Interpreter::with_writer(Box::new(capture.clone()));

    run_line(&mut interpreter, "var x = 21;").expect("declaration should run");
    let value = run_line(&mut interpreter, "x * 2").expect("expression should evaluate");

    assert_eq!(value.map(|v| v.to_string()), Some("42".to_string()));
    assert_eq!(capture.contents(), "");
}

#[test]
fn the_repl_still_reports_real_errors() {
    let capture = Capture::new();
    let mut interpreter = Interpreter::with_writer(Box::new(capture.clone()));

    let error = run_line(&mut interpreter, "print 1 +").expect_err("should fail to parse");
    assert!(error.to_string().contains("Expect expression."));
}
