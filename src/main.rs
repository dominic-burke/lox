use lox::cli::{EX_USAGE, run_file, run_prompt};
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let result = match args.as_slice() {
        [] => run_prompt(),
        [path] => run_file(path),
        _ => {
            eprintln!("Usage: lox [script]");
            return ExitCode::from(EX_USAGE as u8);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(error.exit_code() as u8)
        }
    }
}
