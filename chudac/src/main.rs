//! chudac is a crate that runs the compiler for the chuda programming language
//! requires the lexer, parser, type checker, IR generator, and assembly generator to function

use lexer::LexerError;
use std::env;
use std::path::PathBuf;
use std::process;

mod file_utils;

fn main() {
    let config = Config::build(env::args().collect());
    config.handle_config()
}

/// Prints compiler cli options and usage information
fn print_help() {
    let pad_width = 15;
    println!(
        "Usage: cargo run [options] <source_files>\n\t\
        Where possible options include:\n\t\
        {:<pad_width$} gives a synopsis of useful options\n\t\
        {:<pad_width$} lexes <source_files.chuda/chudi> and produces file_name.lexed for each file\n\t\
        {:<pad_width$} specify where to place generated diagnostic files\n\t\
        {:<pad_width$} specify where to find <source_files>",
        "--help", "--lex", "-D <path>", "-source_path <path>"
    );
    process::exit(1);
}

/// Ensures options are not given after the first source file is seen
fn handle_option_after_file(seen: bool) -> bool {
    if seen {
        eprintln!("Cannot specify options after source files");
        process::exit(1);
    }
    !seen
}

///Stores information about the passed options and parameters
#[derive(Debug)]
struct Config {
    help: bool,
    lex: bool,
    parse: bool,
    code_gen: bool,
    type_check: bool,
    path: PathBuf,
    source_files: Vec<String>,
    source_path: PathBuf,
}

impl Config {
    /// Creates a new config based on the `args`
    ///
    /// # Exits
    ///
    /// if no args are given displays usage information and quits the program
    /// if trouble getting current_dir also exits and displays reason
    /// if an option is passed after the first source file quits and displays why
    fn build(args: Vec<String>) -> Config {
        if args.len() == 1 {
            eprintln!("Problem: Not enough arguments");
            print_help();
        }
        let mut config = Config {
            help: false,
            lex: false,
            parse: false,
            type_check: false,
            code_gen: false,
            path: PathBuf::from(""),
            source_files: vec![],
            source_path: PathBuf::from(""),
        };
        let mut i = 1;
        let mut seen_file = false;
        let mut updated_output_path = false;
        let mut updated_source_path = false;
        while i < args.len() {
            match args[i].as_str() {
                "--help" if handle_option_after_file(seen_file) => {
                    config.help = true;
                }
                "--lex" if handle_option_after_file(seen_file) => {
                    config.lex = true;
                }
                "--lex" if handle_option_after_file(seen_file) => {
                    config.parse = true;
                }
                "-source_path" if handle_option_after_file(seen_file) => {
                    if updated_source_path {
                        eprintln!(
                            "Option: \"-source_path <path>\" was already passed and can only be passed once"
                        );
                        process::exit(1);
                    }
                    updated_source_path = true;
                    i += 1;
                    if i < args.len() {
                        config.source_path = PathBuf::from(&args[i]);
                    }
                }
                "-D" if handle_option_after_file(seen_file) => {
                    if updated_output_path {
                        eprintln!(
                            "Option: \"-D <path>\" was already passed and can only be passed once"
                        );
                        process::exit(1);
                    }
                    updated_output_path = true;
                    i += 1;
                    if i < args.len() {
                        config.path = PathBuf::from(&args[i]);
                    }
                }
                file => {
                    seen_file = true;
                    config.source_files.push(String::from(file));
                }
            }
            i += 1;
        }
        if !updated_output_path {
            config.path = match env::current_dir() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!(
                        "Could not get current directory. Insufficient permissions: {}",
                        e
                    );
                    process::exit(1);
                }
            }
        }
        if !updated_source_path {
            config.path = match env::current_dir() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!(
                        "Could not get current directory. Insufficient permissions: {}",
                        e
                    );
                    process::exit(1);
                }
            }
        }
        config
    }

    /// Parses the config and determines what should be run to complete requested tasks
    ///
    /// # Errors
    ///
    /// `LexerError::InvalidFilesError(files)` if list of source_files contains invalid files
    /// `LexerError::IOReadError(file, error)` if it had trouble reading from `file`
    /// `LexerError::IOWriteError(file, error)` if it had trouble writing to `file`
    fn handle_config(self) {
        if self.source_files.is_empty() || self.help {
            print_help();
        } else if self.lex {
            let files =
                match file_utils::verify_and_construct(&self.source_files, self.path, "lexed") {
                    Ok(files) => files,
                    Err(e) => {
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                };
            let source_texts =
                match file_utils::read_source_files(&self.source_files, self.source_path) {
                    Ok(texts) => texts,
                    Err(v) => {
                        for err in v {
                            eprintln!("{}", err);
                        }
                        process::exit(1);
                    }
                };
            let (tokens, errors) = lexer::lex_files(&source_texts);
            for (mut token_stream, path) in tokens.into_iter().zip(files) {
                let mut file = match file_utils::safe_create_file(&path) {
                    Ok(file) => file,
                    Err(e) => {
                        let file_name = path
                            .as_os_str()
                            .to_str()
                            .unwrap_or("<Could not get file name, it was invalid UTF-8>");
                        eprintln!(
                            "Tried writer to file {} but failed with error {:?}",
                            file_name, e
                        );
                        process::exit(1);
                    }
                };
                if let Err(e) = file_utils::write_deque(&mut token_stream, &mut file) {
                    let file_name = path
                        .as_os_str()
                        .to_str()
                        .unwrap_or("<Could not get file name, it was invalid UTF-8>");
                    eprintln!(
                        "Tried writing to file {}, but failed with error: {:?}",
                        file_name, e
                    );
                    process::exit(1);
                }
            }
            if let Some(errs) = errors {
                for e in errs {
                    match e {
                        LexerError::ErrorToken(msg) => eprintln!("{}", msg),
                    }
                }
                process::exit(1);
            };
        } else if self.parse {
            unimplemented!("I haven't implemented this yet.");
        } else if self.type_check {
            unimplemented!("Type Checking not yet implemented")
        } else if self.code_gen {
            unimplemented!("I haven't implemented this yet either.");
        } else {
            panic!("Somehow you forgot about a config case. Config: {:?}", self)
        }
    }
}
