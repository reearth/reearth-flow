use std::io::{self, BufRead, Write};

use reearth_flow_expr::{compile, eval, Context};

fn main() {
    let ctx = Context::new();
    let stdin = io::stdin();
    print!("> ");
    io::stdout().flush().unwrap();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: {e}");
                break;
            }
        };
        let line = line.trim();
        if !line.is_empty() {
            match compile(line).and_then(|e| eval(&e, &ctx)) {
                Ok(v) => println!("{v}"),
                Err(e) => println!("error: {e}"),
            }
        }
        print!("> ");
        io::stdout().flush().unwrap();
    }
}
