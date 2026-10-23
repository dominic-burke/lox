use lox::cli::{run_file, run_prompt};
use std::env;

const EX_USAGE: i32 = 64;
const EX_DATAERR: i32 = 1;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: lox <script>");
        std::process::exit(EX_USAGE);
    } else if args.len() == 2 {
        if let Err(e) = run_file(&args[1]) {
            eprintln!("Error: {}", e);
            std::process::exit(EX_DATAERR);
        }
    } else {
        if let Err(e) = run_prompt() {
            eprintln!("Error: {}", e);
            std::process::exit(EX_DATAERR);
        }
    }
}
