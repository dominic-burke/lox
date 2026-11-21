# lox (Rust)

A tree-walk interpreter for [Lox](https://craftinginterpreters.com/the-lox-language.html),
the language from Robert Nystrom's *Crafting Interpreters*, written in Rust.

**This is a partial Lox implementation.** It covers the language through
functions and closures (chapters 1–10 of the book). See [Status](#status) 
for the details.

## Building and running

Requires Rust 1.85 or newer (the crate uses edition 2024), and no external
dependencies.

```sh
cargo build            # debug binary at target/debug/lox
cargo build --release

./target/debug/lox program.lox   # run a script
./target/debug/lox               # start the REPL
cargo run -- program.lox         # same thing, through cargo
```

The REPL keeps its globals between lines and echoes the value of a bare
expression, so it doubles as a calculator:

```
> var x = 21;
> x * 2
42
> print "hello";
hello
```

Exit codes follow the `sysexits.h` conventions used throughout the book:

| Code | Meaning |
| ---- | ------- |
| 0    | success |
| 64   | usage error (too many arguments) |
| 65   | scan or compile error |
| 70   | runtime error |
| 74   | I/O error (e.g. script not found) |

## Example

```lox
fun makeCounter() {
  var i = 0;
  fun count() { i = i + 1; return i; }
  return count;
}

var counter = makeCounter();
print counter(); // 1
print counter(); // 2

fun fib(n) {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2);
}
print fib(10); // 55
```

## Status

### Implemented

- Scanning: comments (`//` and `/* */`), strings (no escape sequences, as in
  the book), numbers, identifiers, all single/double-character punctuators, and
  keywords.
- Expressions: arithmetic, comparison, equality, logical `and`/`or`
  (short-circuiting), unary `!`/`-`/`+`, grouping, string concatenation,
  assignment, variables, function calls.
- Statements: `print`, expression statements, `var` declarations, blocks,
  `if`/`else`, `while`, `for` (desugared to `while`), `fun`, `return`.
- Semantics: lexical scoping with an environment chain, closures, recursion,
  and the native `clock()` function. Numbers are IEEE-754 doubles; `nil` and
  `false` are the only falsey values.
- Errors: scanner, parser, and runtime errors all report a line number.

### Not implemented

- **Classes, instances, methods, `this`, `super`, inheritance.** The scanner
  does not even have a `class` keyword, so `this` and `super` are currently
  unused tokens.
- **The resolver (static variable resolution).** Closures look variables up at
  call time instead of resolving them at compile time. The practical
  consequence: a function declared inside a block sees variables declared
  later in that same block, and duplicate declarations in one scope are not
  rejected.
- **Parser error recovery.** Parsing stops at the first error; the scanner
  still reports every lexical error it finds.
- Anything else in the book: bytecode VM, garbage collection, `break`/`continue`.

## Tests

```sh
cargo test --all-targets   # unit + end-to-end tests
cargo clippy --all-targets
cargo fmt --check
```

The test suite has 41 tests: the scanner's unit tests plus end-to-end tests in
`tests/interpreter.rs` that run source through the public API and assert on
printed output and error text.

The interpreter also runs the official
[Crafting Interpreters test suite](https://github.com/munificent/craftinginterpreters/tree/master/test)
at **134/161** for the chapters it implements. The 27 failures are all accounted
for, and each is a feature listed above that this implementation does not have:

| Count | Cause |
| ----- | ----- |
| 13 | class tests — no `class` in this Lox |
| 7  | resolver tests (early binding, duplicate declarations) |
| 2  | parser error-recovery tests (the book reports several errors per file) |
| 2  | expression-mode test files that need the book's AST dump runner |
| 3  | error-message wording (two need property access grammar) |

## Layout

| File | Contents |
| ---- | -------- |
| `src/token.rs` | `Token`, `TokenType`, `TokenError` |
| `src/scanner.rs` | Source text to tokens, with lexical errors |
| `src/parser.rs` | Tokens to `Stmt`/`Expression` trees, `ParseError` |
| `src/value.rs` | `Value`, the `LoxCallable` trait, `RuntimeError` |
| `src/environment.rs` | `Rc<RefCell<..>>` scope chain |
| `src/interpreter.rs` | Evaluator, `LoxFunction`, native `clock()` |
| `src/cli.rs` | Run a file / REPL, error type and exit codes |
| `src/main.rs` | Argument handling |

## Known limits

- Recursion deeper than roughly a thousand Lox calls overflows the native stack
  and aborts the process; there is no call-depth guard (jlox behaves the same
  way). A guard or a larger-stack thread would fix it.
- Each source line in the REPL is parsed on its own, so multi-line function and
  block definitions aren't supported there. Scripts have no such restriction.
