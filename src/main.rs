use std::{
    env,
    env::VarError,
    fs,
    fs::File,
    io::{self, stdin, BufRead, BufReader, Error, ErrorKind},
    path::{Path, PathBuf},
    process::Command,
    str,
};

use clap::Parser;
use dialoguer::{console::Term, theme::ColorfulTheme, Select};

#[derive(Parser)]
struct Cli {
    /// Sequence of search terms used to select matching lines
    pattern: Vec<String>,
    /// Select a history file to search from home folder
    #[arg(long, default_value_t = false)]
    history: bool,
    /// Select a complete file path to search from
    #[arg(short, long, default_value_t = false)]
    file: bool,
}

struct History {
    home_path: PathBuf,
    shell_type: Result<String, ShellError>,
    history_path: PathBuf,
    history_list: Vec<String>,
    query_results: Vec<String>,
}

impl History {
    fn new() -> Self {
        let shell_type = get_shell();

        let home_var = env::var("HOME").unwrap_or_else(|e| {
            println!("{} ${}", e, "HOME");
            String::new()
        });

        let home_path = PathBuf::from(home_var);
        let mut history_path = home_path.clone();
        if shell_type.is_ok() {
            history_path.push(&format!(".{}_history", shell_type.as_ref().unwrap()));
        }
        History {
            home_path,
            shell_type,
            history_path,
            history_list: Vec::new(),
            query_results: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum ShellError {
    EnvVarError(std::env::VarError),
    CommandError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ShellError::EnvVarError(e) => write!(f, "Failed to get environment variable: {}", e),
            ShellError::CommandError(e) => write!(f, "Command execution failed: {}", e),
            ShellError::Utf8Error(e) => write!(f, "Failed to decode output: {}", e),
        }
    }
}

impl std::error::Error for ShellError {}

fn get_shell() -> Result<String, ShellError> {
    let shell_path = match env::var("SHELL") {
        Ok(shell) => Ok(shell),
        Err(e) => Err(ShellError::EnvVarError(e)),
    }
    .or_else(|_| {
        let output = Command::new("sh")
            .arg("-c")
            .arg("echo $0")
            .output()
            .map_err(ShellError::CommandError)?;

        if output.status.success() {
            let shell = str::from_utf8(&output.stdout).map_err(ShellError::Utf8Error)?;
            Ok(shell.trim().to_string())
        } else {
            Err(ShellError::CommandError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Command execution was not successful",
            )))
        }
    });

    match shell_path {
        Ok(path) => {
            let shell = path.split("/").last().unwrap_or("").to_string();
            Ok(shell)
        }
        Err(err) => Err(err),
    }
}

impl History {
    fn get_hist_file_paths(&self) -> Result<Vec<PathBuf>, VarError> {
        let paths = fs::read_dir(&self.home_path).unwrap();
        let mut hist_paths: Vec<PathBuf> = Vec::new();

        for entry in paths {
            let path = entry.unwrap();
            if path.file_name().to_string_lossy().contains("history") {
                hist_paths.push(path.path());
            }
        }

        Ok(hist_paths)
    }

    fn read_lines<P>(file_path: P) -> io::Result<io::Lines<BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(file_path)?;
        Ok(BufReader::new(file).lines())
    }

    fn load_history(&mut self) {
        match Self::read_lines(&self.history_path) {
            Ok(lines) => {
                for line in lines {
                    match line {
                        Ok(line_str) => self.history_list.push(line_str),
                        Err(e) => eprintln!("Failed to read line: {}", e),
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read lines from file: {}", e);
            }
        }
    }
    fn query_history(&mut self) {
        let args = Cli::parse();
        println!(
            "Searching for - {:?} - in {:?}",
            args.pattern, self.history_path
        );

        for line in &self.history_list {
            let mut line = line.clone();

            match &self.shell_type {
                Ok(shell_type) => {
                    if shell_type == "zsh" {
                        line = line.split(';').last().unwrap_or("").to_string();
                    }
                }
                Err(e) => {
                    eprintln!("Shell type error: {}", e);
                    return;
                }
            }
            let found = args.pattern.iter().all(|arg| line.contains(arg));
            if found {
                self.query_results.push(line.clone());
                println!("{}", line);
            }
        }
    }
}
fn choose_file(items: Vec<PathBuf>) -> Result<PathBuf, Error> {
    let items_display: Vec<String> = items
        .iter()
        .map(|path| path.display().to_string())
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items_display)
        .default(0)
        .interact_on_opt(&Term::stderr())?;

    match selection {
        Some(index) => {
            println!("User selected item : {}", items_display[index]);
            Ok(items[index].to_owned())
        }
        None => {
            println!("User did not select anything");
            Err(Error::new(ErrorKind::Other, "No selection was made"))
        }
    }
}

fn get_file_path() -> PathBuf {
    println!("Please enter a valid file path:");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");
    PathBuf::from(input)
}

fn main() {
    let args = Cli::parse();
    let mut history = History::new();
    let search_path: PathBuf;
    if args.history {
        let paths = history
            .get_hist_file_paths()
            .expect("Failed to history paths.");
        search_path = choose_file(paths).expect("Failed to choose a path.");
    } else if args.file {
        search_path = get_file_path()
        // search
    } else if history.history_path.exists() {
        println!(
            "It appears that you are using {} shell as your default.",
            history.shell_type.as_ref().unwrap()
        );
        search_path = history.history_path
    } else {
        search_path = get_file_path()
    }
    history.history_path = search_path;
    history.load_history();
    history.query_history();
}
