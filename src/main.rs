use std::{
    // borrow::Borrow,
    collections::HashMap,
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
    search_terms: Vec<String>,
    /// Select a history file to search from home folder
    #[arg(long)]
    history: bool,
    /// Select a complete file path to search from
    #[arg(short, long)]
    file: bool,
    /// Dedupe output lines
    #[arg(short, long)]
    dedupe: bool,
    /// Order dependent parsing
    #[arg(short, long)]
    ordered: bool,
}

struct History {
    // The list of search terms captured from command line args.
    search_terms: Vec<String>,
    // The full path to the home directory.
    home_path: PathBuf,
    // The short name of the default shell.
    shell_type: Result<String, ShellError>,
    // The full path to the file to be searched.
    history_path: PathBuf,
    // List of full paths to history files in home folder.
    history_list: Vec<String>,
    // Map of unique file lines to line_numbers.
    history_map: HashMap<String, Vec<u16>>,
    // List of matching lines from search file.
    query_results: Vec<String>,
    // Function to match search_terms with.
    match_fn: fn(String, Vec<String>) -> bool,
}

impl History {
    fn new() -> Self {
        let args = Cli::parse();
        let search_terms = args.search_terms;

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
            search_terms,
            home_path,
            shell_type,
            history_path,
            history_list: Vec::new(),
            history_map: HashMap::new(),
            query_results: Vec::new(),
            match_fn: unordered_match,
        }
    }

    fn apply_flags(&mut self) {
        let matches = Cli::parse();

        if matches.history {
            let paths = self
                .get_hist_file_paths()
                .expect("Failed to history paths.");
            self.history_path = choose_file(paths).expect("Failed to choose a path.");
        } else if matches.file {
            self.history_path = get_file_path();
        } else if matches.ordered {
            // set History::match_fn
            self.match_fn = ordered_match;
        } else {
            // default behavior
        }
    }
    fn get_hist_file_paths(&self) -> Result<Vec<PathBuf>, VarError> {
        let paths = fs::read_dir(&self.home_path).unwrap();
        let mut hist_paths: Vec<PathBuf> = Vec::new();

        for p in paths {
            let path = p.unwrap();
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

    fn query_history_map(&mut self) {
        println!(
            "Searching for - {:?} - in {:?}",
            self.search_terms, self.history_path
        );

        for key in self.history_map.keys() {
            let found = (self.match_fn)(key.clone(), self.search_terms.clone());
            if found {
                self.query_results.push(key.clone());
                if let Some(value) = self.history_map.get(key) {
                    println!("{:?}:: {}", value, key);
                }
            }
        }
    }

    fn load_history_map(&mut self) {
        let mut index: u16 = 0;
        for line in self.history_list.clone() {
            index += 1;
            self.history_map
                .entry(line)
                .or_insert_with(Vec::new)
                .push(index);
        }
    }
}

#[derive(Debug)]
enum ShellError {
    EnvVar(std::env::VarError),
    Command(std::io::Error),
    Utf8(std::str::Utf8Error),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ShellError::EnvVar(e) => write!(f, "Failed to get environment variable: {}", e),
            ShellError::Command(e) => write!(f, "Command execution failed: {}", e),
            ShellError::Utf8(e) => write!(f, "Failed to decode output: {}", e),
        }
    }
}

impl std::error::Error for ShellError {}

fn get_shell() -> Result<String, ShellError> {
    let shell_path = match env::var("SHELL") {
        Ok(shell) => Ok(shell),
        Err(e) => Err(ShellError::EnvVar(e)),
    }
    .or_else(|_| {
        let output = Command::new("sh")
            .arg("-c")
            .arg("echo $0")
            .output()
            .map_err(ShellError::Command)?;

        if output.status.success() {
            let shell = str::from_utf8(&output.stdout).map_err(ShellError::Utf8)?;
            Ok(shell.trim().to_string())
        } else {
            Err(ShellError::Command(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Command execution was not successful",
            )))
        }
    });

    match shell_path {
        Ok(path) => {
            let shell = path.split('/').last().unwrap_or("").to_string();
            Ok(shell)
        }
        Err(err) => Err(err),
    }
}

fn get_current_directory() -> PathBuf {
    let res = env::current_dir();
    match res {
        Ok(path) => {
            let p = path.into_os_string().into_string().unwrap();
            return PathBuf::from(p);
        }
        Err(_) => PathBuf::from(""),
    }
}


/// Functions ordered_match and unordered_match contain respective
/// matching logic that corresponds to the `--order` flag. Either
/// function is set to the `History::match_fn` attribute.

/// Sequentially check for `key` text for a match with the arg terms.
fn ordered_match(mut key: String, args: Vec<String>) -> bool {
    // let mut line = key;
    for a in args {
        if let Some(pos) = key.find(&a) {
            key = key[pos + a.len()..].to_string();
        } else {
            return false;
        }
    }
    true
}

/// Check that all arg terms match the text phrase.
fn unordered_match(mut key: String, args: Vec<String>) -> bool {
    for a in args {
        if let Some(_) = key.find(&a) {
            key = key.replacen(&a, "", 1);
        } else {
            return false;
        }
    }
    true
}

fn get_file_path() -> PathBuf {
    println!("Please enter a valid file path:");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");
    let trimmed_input = input.trim_end_matches('\n'); // Trim the newline at the end
    PathBuf::from(trimmed_input)
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

fn main() {
    let mut history = History::new();

    history.apply_flags();
    history.load_history();
    history.load_history_map();
    history.query_history_map();

    println!("history_list = {}", history.history_list.len());
    println!("history_map = {}", history.history_map.keys().len());
}
