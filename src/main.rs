use std::{
    env,
    env::VarError,
    fs,
    fs::File,
    io::{self, stdin, BufRead, BufReader, Error, ErrorKind},
    path::{Path, PathBuf},
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
    shell_type: String,
    history_path: PathBuf,
    history_list: Vec<String>,
    query_results: Vec<String>,
}

impl History {
    fn new() -> Self {
        let shell_type = Self::get_str_from_env("SHELL")
            .split("/")
            .last()
            .unwrap_or_else(|| "")
            .to_string();

        let home_path = Self::get_path_from_env("HOME");
        let mut history_path = home_path.clone();
        history_path.push(&format!(".{}_history", shell_type));

        History {
            home_path,
            shell_type,
            history_path,
            history_list: Vec::new(),
            query_results: Vec::new(),
        }
    }

    fn get_str_from_env(env_var: &str) -> String {
        env::var(env_var).unwrap_or_else(|e| {
            println!("{} ${}", e, env_var);
            String::new()
        })
    }

    fn get_path_from_env(env_var: &str) -> PathBuf {
        PathBuf::from(Self::get_str_from_env(env_var))
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
        let file_data = Self::read_lines(&self.history_path);
        if let Ok(data) = file_data {
            for line in data {
                if let Ok(line_str) = line {
                    self.history_list.push(line_str)
                }
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
                    let mut found = false;
                for arg in &args.pattern {
                    if line.contains(&*arg) {
                        found = true;
                    } else {
                        found = false;
                        break;
                    }
                }
                if found == true {
                    self.query_results.push(line.to_owned());
                    println!("{}", line.to_owned());
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
    } else if history.shell_type != "" {
        println!(
            "It appears that you are using {} shell as your default.",
            history.shell_type
        );
        search_path = history.history_path
    } else {
        search_path = get_file_path()
    }
    history.history_path = search_path;
    history.load_history();
    history.query_history();
}
