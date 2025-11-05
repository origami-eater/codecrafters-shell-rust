#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    print!("$ ");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    print!("{}: command not found", input);

    io::stdout().flush().unwrap();
}
