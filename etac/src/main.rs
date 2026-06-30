use std::env;
use std::process;
use lexer::lexer;

fn main() {
    let config = Config::build(env::args().collect());
    config.handle_config()
}

#[derive(Debug)]
struct Config{
    help : bool,
    lex : bool,
    parse : bool,
    code_gen : bool,
    path : String,
    source_files : Vec<String>
}

impl Config{
    fn build(args : Vec<String>) -> Config{
        if args.len() == 1{
            eprintln!("Problem: Not enough arguments");
            print_help();
        }
        let mut config = Config{
            help:false,
            lex:false,
            parse:false,
            code_gen:false,
            path:String::from(""), 
            source_files: vec![]
        };
        for mut i in 1 .. args.len(){
            match args[i].as_str() {
                "--help" => {
                    config.help = true;
                }
                "--lex" => {
                    config.lex = true;
                }
                "-D" => {
                    i += 1;
                    if i < args.len(){
                        config.path = args[i].clone();
                    }
                }
                file =>{
                    config.source_files.push(String::from(file));
                }
            }
        }
        return config;
    }

    fn handle_config(&self){
        if self.source_files.is_empty() || self.help{
            print_help();
        }
        else if self.lex{
            if let Err(value) = lexer::lex_files(&self.source_files, &self.path){
                match value{
                    lexer::LexerError::InvalidFilesError(files) => {
                            eprintln!("Invalid files given: {:?}", files);
                        eprintln!("Please ensure files given have ending \".eti\" or \".eta\"");
                        process::exit(1);
                    }
                    lexer::LexerError::IOError(file, error) => {
                        eprintln!("Tried reading file: {}, but failed with error: {}", file, error);
                        process::exit(1);
                    }
                }
            }

        }
        else if self.parse{
            unimplemented!("I haven't implemented this yet.");
        }
        else if self.code_gen{
            unimplemented!("I haven't implemented this yet either.");
        }
        else{
            panic!("Somehow you forgot abotu a config case. Config: {:?}", self)
        }
    }
}

fn print_help(){
    let pad_width = 15;
    println!("Usage: cargo run [options] <source_files>\n\t\
                Where possible options include:\n\t\
                {:<pad_width$} gives a synopsis of useful options\n\t\
                {:<pad_width$} lexes filename.eta and produces filename.lexed\n\t\
                {:<pad_width$} specify where to place generated diagnostic files"
                , "--help", "--lex", "-D <path>"
            );
    process::exit(1);
}
