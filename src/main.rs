use std::env;
use lox::{run_prompt, run_file};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        eprintln!("Usage: lox <script>");
        std::process::exit(64);
    } else if args.len() == 2 {
        if let Err(e) = run_file(&args[1]) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else {
        if let Err(e) = run_prompt() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
