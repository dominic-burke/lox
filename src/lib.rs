mod scanner;
mod token;
use std::io::Write;

pub fn run_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(path)?;
    run(source);
    Ok(())
}

pub fn run_prompt() -> Result<(), Box<dyn std::error::Error>> {
    let mut line = String::new();
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        line.clear();
        if std::io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        run(line.clone());
    }
    Ok(())
}

pub fn run(source: String) {
    let mut scanner = scanner::Scanner::new(source);
    let tokens = scanner.scan_tokens();
    for token in tokens {
        println!("{:?}", token);
    }
}
