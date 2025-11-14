use std::env;
use std::fmt;
use std::fs;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
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

#[derive(Debug)]
enum ShellError {
    CommandNotFound(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::CommandNotFound(cmd) => write!(f, "{}: command not found", cmd),
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

fn get_system_executable_path(name: &str) -> Option<String> {
    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            if let Some(absolute_path) = Path::new(&path).join(name).canonicalize().ok() {
                if let Ok(metadata) = fs::metadata(&absolute_path) {
                    let perms = metadata.permissions();
                    if perms.mode() & 0o111 != 0 {
                        //rwx
                        // TODO: see what the approach to reduce nesting is in rust
                        return absolute_path.to_str().map(|x| x.to_string());
                    }
                }
            }
        }
    }
    return None;
}

fn main() {
    let handlers = get_handlers();

    let run_command_or_error = |command: &Command| -> Option<ShellError> {
        match &command.id {
            id if id == "type" => {
                if let Some(query) = command.args.first() {
                    if handlers.contains_key(query) || query == "type" {
                        println!("{} is a shell builtin", query);
                    } else if let Some(exe_path) = get_system_executable_path(&query) {
                        println!("{} is {}", query, exe_path);
                    } else {
                        println!("{}: not found", query);
                    }
                    return None;
                }
            }
            _ => {}
        }

        if let Some(handler) = handlers.get(&command.id) {
            handler(&command);
            return None;
        }

        Some(ShellError::CommandNotFound(command.id.clone()))
    };

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if let Some(command) = Command::parse(input) {
            match run_command_or_error(&command) {
                Some(error) => eprintln!("{}", error),
                None => {}
            }
        }
    }
}
