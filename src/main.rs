use std::env;
use std::fmt;
use std::fs;
use std::fs::Permissions;
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

struct Executable {
    path: String,
    permissions: Permissions,
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
        } else {
            exit(0);
        }
    });

    executor_map.insert(String::from("echo"), |command| {
        println!("{}", command.args.join(" "));
    });

    executor_map
}

fn get_executable(name: &str) -> Option<Executable> {
    if let Some(paths) = env::var_os("PATH") {
        for dir in env::split_paths(&paths) {
            let abs_path = match Path::new(&dir).join(name).canonicalize() {
                Ok(abs_path) => abs_path,
                _ => continue,
            };
            let metadata = fs::metadata(&abs_path).ok()?;
            let perms = metadata.permissions();
            if perms.mode() & 0o111 != 0 {
                return abs_path.to_str().map(|x| Executable {
                    path: x.to_owned(),
                    permissions: perms,
                });
            } else {
                continue;
            }
        }
    }
    return None;
}

fn run_process(command: &Command, executable: &Executable) {
    let process = match std::process::Command::new(&command.id)
        .args(command.args.clone())
        .spawn()
    {
        Ok(process) => process,
        Err(err) => panic!("encountered error spawning a process: {}", err),
    };

    let output = match process.wait_with_output() {
        Ok(output) => output,
        Err(err) => panic!("Retrieving output error: {}", err),
    };

    let stdout = match std::string::String::from_utf8(output.stdout) {
        Ok(stdout) => stdout,
        Err(err) => panic!("Translating output error: {}", err),
    };

    print!("{}", stdout);
}

fn main() {
    let handlers = get_handlers();

    let run_command_or_error = |command: &Command| -> Option<ShellError> {
        match &command.id {
            id if id == "type" => {
                if let Some(query) = command.args.first() {
                    if handlers.contains_key(query) || query == "type" {
                        println!("{} is a shell builtin", query);
                    } else if let Some(executable) = get_executable(&query) {
                        println!("{} is {}", query, executable.path);
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
        } else if let Some(executable) = get_executable(&command.id) {
            run_process(command, &executable);
            return None;
        } else {
            return Some(ShellError::CommandNotFound(command.id.clone()))
        }
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
