use std::io::stdout;
#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");
    stdout().flush().unwrap();

    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();

    let result: Vec<&str> = input.split_whitespace().collect();

    if let [cmd] = result.as_slice() {
        match cmd {
            _ => println!("{}: command not found", cmd),
        }
    }
}
