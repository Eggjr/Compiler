use lexer;
use lexer::LexerError;
use std::env;
use std::io;
use std::path::PathBuf;
use std::process;

fn main() {
    let config = Config::build(env::args().collect());
    config.handle_config()
}

#[derive(Debug)]
struct Config {
    help: bool,
    lex: bool,
    parse: bool,
    code_gen: bool,
    path: PathBuf,
    source_files: Vec<String>,
}

impl Config {
    fn build(args: Vec<String>) -> Config {
        if args.len() == 1 {
            eprintln!("Problem: Not enough arguments");
            print_help();
        }
        let mut config = Config {
            help: false,
            lex: false,
            parse: false,
            code_gen: false,
            path: match env::current_dir() {
                io::Result::Ok(path) => path,
                io::Result::Err(e) => {
                    eprintln!("Issue with specified path: {}", e);
                    process::exit(1);
                }
            },
            source_files: vec![],
        };
        let mut i = 0;
        let mut seen_file = false;
        while i < args.len() {
            match args[i].as_str() {
                "--help" if handle_option_after_file(seen_file) => {
                    config.help = true;
                }
                "--lex" if handle_option_after_file(seen_file) => {
                    config.lex = true;
                }
                "-D" if handle_option_after_file(seen_file) => {
                    if seen_file {
                        eprintln!("Cannot specify options after source files");
                        process::exit(1);
                    }
                    i += 1;
                    if i < args.len() {
                        config.path = PathBuf::from(args[i].clone());
                    }
                }
                file => {
                    seen_file = true;
                    config.source_files.push(String::from(file));
                }
            }
            i += 1;
        }
        return config;
    }

    fn handle_config(self) {
        if self.source_files.is_empty() || self.help {
            print_help();
        } else if self.lex {
            let tokens_paths = match lexer::lex_files(
                self.source_files,
                self.path,
            ) {
                Err(errors) => {
                    for e in errors {
                        match e {
                            LexerError::InvalidFilesError(files) => {
                                eprintln!("Invalid files given: {:?}", files);
                                eprintln!(
                                    "Please ensure files given have ending \".eti\" or \".eta\""
                                );
                            }
                            LexerError::IOError(file, error) => eprintln!(
                                "Tried reading file: {}, but failed with error: {}",
                                file, error
                            ),
                            LexerError::ErrorToken(msg) => eprintln!("{}", msg),
                        }
                    }
                    process::exit(1);
                }
                Ok(t) => t,
            };
            for (mut token_stream, path) in tokens_paths {
                if let Err(e) = lexer::write_to_file(&mut token_stream, path) {
                    eprintln!("{:?}", e);
                    process::exit(1);
                }
            }
        } else if self.parse {
            unimplemented!("I haven't implemented this yet.");
        } else if self.code_gen {
            unimplemented!("I haven't implemented this yet either.");
        } else {
            panic!("Somehow you forgot about a config case. Config: {:?}", self)
        }
    }
}

fn print_help() {
    let pad_width = 15;
    println!(
        "Usage: cargo run [options] <source_files>\n\t\
        Where possible options include:\n\t\
        {:<pad_width$} gives a synopsis of useful options\n\t\
        {:<pad_width$} lexes filename.eta and produces filename.lexed\n\t\
        {:<pad_width$} specify where to place generated diagnostic files",
        "--help", "--lex", "-D <path>"
    );
    process::exit(1);
}

fn handle_option_after_file(seen: bool) -> bool {
    if seen {
        eprintln!("Cannot specify options after source files");
        process::exit(1);
    }
    !seen
}
