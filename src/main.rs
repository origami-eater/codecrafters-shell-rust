#[allow(unused_imports)]
use std::io::{self, Write};
use std::{collections::HashMap, process::exit};

struct Command {
    id: String,
    args: Vec<String>,
}

impl Command {
    fn parse(input: String) -> Option<Command> {
        let tokens: Vec<String> = input
            .trim()
            .split(' ')
            .map(|token| token.trim().to_owned())
            .filter(|token| !token.is_empty())
            .collect();

        match tokens.as_slice() {
            [id, args @ ..] => Some(Command {
                id: id.clone(),
                args: args.to_vec(),
            }),
            _ => None,
        }
    }
}

type CommandHandler = fn(&Command) -> ();

fn get_handlers() -> HashMap<String, CommandHandler> {
    let mut executor_map: HashMap<String, CommandHandler> = HashMap::new();

    executor_map.insert(String::from("exit"), |command| {
        if let Some(exit_code) = command.args.first().and_then(|x| x.parse().ok()) {
            exit(exit_code);
        }
    });

    executor_map.insert(String::from("echo"), |command| {
        println!("{}", command.args.join(" "));
    });

    executor_map
}

fn main() {
    let handlers = get_handlers();
    
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if let Some(command) = Command::parse(input) {
            if let Some(handler) = handlers.get(&command.id) {
                handler(&command);
            } else {
                println!("{}: command not found", command.id);
            }
        }
    }
}
