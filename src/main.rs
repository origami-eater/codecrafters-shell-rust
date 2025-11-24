use anyhow;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::exit;
use std::process::Stdio;

#[derive(Debug)]
enum ShellError {
    InvalidArgs(&'static str),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::InvalidArgs(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ShellError {}

type ShellResult = Result<String, anyhow::Error>;

struct Command {
    name: String,
    args: Vec<String>,
}

impl Command {
    fn parse(input: &str) -> Option<Command> {
        let tokens: Vec<&str> = input.trim().split_whitespace().collect();

        if let Some((name, args)) = tokens.split_first() {
            return Some(Command {
                name: name.to_string(),
                args: args.iter().map(|x| x.to_string()).collect(),
            });
        } else {
            return None;
        }
    }
}

enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
}

// manually defining it since ideally the least external dependencies
impl Builtin {
    fn from_str(s: &str) -> Option<Builtin> {
        match s {
            "echo" => Some(Builtin::Echo),
            "exit" => Some(Builtin::Exit),
            "type" => Some(Builtin::Type),
            "pwd" => Some(Builtin::Pwd),
            _ => None,
        }
    }
}

fn run_builtin(b: &Builtin, args: &[String]) -> ShellResult {
    return match b {
        Builtin::Echo => Ok(args.join(" ")),
        Builtin::Exit => {
            if let Some(exit_code) = args.first().and_then(|x| x.parse().ok()) {
                exit(exit_code);
            } else {
                exit(0);
            }
        }
        Builtin::Type => {
            let query = args
                .first()
                .ok_or(ShellError::InvalidArgs("type requires an argument"))?;

            return if let Some(_) = Builtin::from_str(query) {
                Ok(format!("{} is a shell builtin", query))
            } else if let Some(sys_exec) = SystemExecutable::from_str(&query) {
                Ok(format!("{} is {}", query, sys_exec.abs_path))
            } else {
                Ok(format!("{}: not found", query))
            };
        }
        Builtin::Pwd => match env::current_dir()?.to_str() {
            Some(cwd) => Ok(cwd.to_owned()),
            None => Err(anyhow::anyhow!("could not parse current directory path")),
        },
    };
}

struct SystemExecutable {
    name: String,
    abs_path: String,
}

impl SystemExecutable {
    fn from_str(exec_name: &str) -> Option<SystemExecutable> {
        let paths = env::var_os("PATH")?;

        return env::split_paths(&paths).find_map(|dir| {
            let abs_path = Path::new(&dir).join(exec_name).canonicalize().ok()?;
            let perms = fs::metadata(&abs_path).ok()?.permissions();

            if perms.mode() & 0o111 != 0 {
                return abs_path.to_str().map(|x| SystemExecutable {
                    name: exec_name.to_owned(),
                    abs_path: x.to_owned(),
                });
            } else {
                None
            }
        });
    }
    fn run_foreground(&self, args: &[String]) -> ShellResult {
        let process = std::process::Command::new(&self.name)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(args)
            .spawn()?;

        let output = process.wait_with_output()?;
        let stdout = std::string::String::from_utf8(output.stdout)?;
        Ok(stdout.trim().to_owned())
    }
}

fn process_command(cmd: &Command) -> ShellResult {
    return if let Some(builtin) = Builtin::from_str(&cmd.name) {
        run_builtin(&builtin, &cmd.args)
    } else if let Some(sys_exec) = SystemExecutable::from_str(&cmd.name) {
        sys_exec.run_foreground(&cmd.args)
    } else {
        Ok(format!("{}: command not found", &cmd.name))
    };
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if let Some(cmd) = Command::parse(&input) {
            match process_command(&cmd) {
                Ok(out) => println!("{}", out),
                Err(err) => println!("{}", err),
            }
        }
    }
}
