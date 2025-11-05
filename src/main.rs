#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let sanitized_input = input.trim();

        if !sanitized_input.is_empty() {
            println!("{}: command not found", sanitized_input);
        }
    }
}
