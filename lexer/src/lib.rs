//! # Lexer
//!
//! `Lexer` is a crate for lexing the eta programming language
use std::collections::VecDeque;
use std::path::PathBuf;

mod lexer_error;
mod token;
mod token_scanner;
mod token_type;
mod tokenizer;

pub use lexer_error::LexerError;
pub use token::Token;
pub use token_type::TokenType;
pub use tokenizer::Tokenizer;

/// Checks if `source_files` only contains file names with *.eta* or *.eti* endings
///
/// # Errors
///
/// returns `LexerError::InvalidFilesError(invalids)` where `invalids` is a list of files with non *.eta* or *.eti* endings
fn verify_files(source_files: &[String]) -> Result<(), LexerError> {
    let invalids: Vec<String> = source_files
        .iter()
        .filter(|file| !file.ends_with(".eta") && !file.ends_with(".eti"))
        .cloned()
        .collect();
    if !invalids.is_empty() {
        return Err(LexerError::InvalidFilesError(invalids));
    }
    Ok(())
}

/// Constructs output `.lexed` files with the specified prefix `path` for each file name in `source_files`
///
/// # Requires:
///
/// All files in `source_files` end with `.eta` or `.eti`
fn construct_paths(source_files: &[String], path: PathBuf) -> Vec<PathBuf> {
    source_files
        .iter()
        .map(|file_name| path.join(file_name).with_extension("lexed"))
        .collect()
}

/// Writes `tokens` to specified `writer` for diagnostic purposes
///
/// # Examples
///
/// ```
///
/// use lexer::Token;
/// use lexer::TokenType;
/// use std::collections::VecDeque;
///
/// # fn main(){
/// let mut buffer = vec![];
/// let mut tokens : VecDeque<Token> = VecDeque::new();
/// tokens.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
/// tokens.push_back(Token::new(1, 6, TokenType::Colon));
/// tokens.push_back(Token::new(1, 7, TokenType::Int));
/// tokens.push_back(Token::new(1, 10, TokenType::Assign));
/// tokens.push_back(Token::new(1, 11, TokenType::Integer(10)));
/// lexer::write_tokens(&mut tokens, &mut buffer).expect("Writing failed");
/// assert_eq!(String::from_utf8(buffer).expect("This is valid utf-8"), "1:1 id Pizza\n1:6 :\n1:7 int\n1:10 =\n1:11 integer 10\n");
/// # }
/// ```
///
/// # Errors
///
/// returns `LexerError::IOWriteError()` if it failed to write to `writer`
pub fn write_tokens<W: std::io::Write>(
    tokens: &mut VecDeque<Token>,
    writer: &mut W,
) -> Result<(), LexerError> {
    while let Some(token) = tokens.pop_front() {
        if let Err(e) = writeln!(writer, "{}", token) {
            return Err(LexerError::IOWriteError(e));
        };
    }
    Ok(())
}

/// Lexes `source_files` and returns a token representation of the contents with the corresponding
/// `path`/file_name.lexed output file
///
/// # Examples
///```
/// use std::path::PathBuf;
/// use std::collections::VecDeque;
/// use lexer::TokenType;
/// use lexer::Token;
/// use std::env;
/// # fn main(){
/// let mut tokens : VecDeque<Token> = VecDeque::new();
/// tokens.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
/// tokens.push_back(Token::new(1, 6, TokenType::Colon));
/// tokens.push_back(Token::new(1, 7, TokenType::Int));
/// tokens.push_back(Token::new(1, 10, TokenType::Assign));
/// tokens.push_back(Token::new(1, 11, TokenType::Integer(10)));
///
/// let cur_dir = env::current_dir().expect("Current directory does not exists");
///
/// let tokens_and_paths = match lexer::lex_files(vec![String::from("../eta_programs/lexer_files/lex_test_3.eta")], cur_dir.clone()){
///     Ok(v) => v,
///     Err(e) => {
///         dbg!(e);
///         panic!("Lexing Failed")
///     },
/// };
/// dbg!(&tokens_and_paths);
/// assert_eq!(tokens_and_paths, vec![(tokens, cur_dir.join("../eta_programs/lexer_files/lex_test_3.lexed"))]);
/// # }
/// ```
///
/// # Errors
///
/// returns `Vec<LexerError>` upon failure to verify files, or upon termination if any files had
/// an error token written to them, detailing where the errors can be viewed and giving a brief
/// description of the errors
pub fn lex_files(
    source_files: Vec<String>,
    path: PathBuf,
) -> Result<Vec<(VecDeque<Token>, PathBuf)>, Vec<LexerError>> {
    let mut err_vec = vec![];
    if let Err(e) = verify_files(&source_files) {
        err_vec.push(e);
        return Err(err_vec);
    };
    let mut token_queues: Vec<(VecDeque<Token>, PathBuf)> = vec![];
    let output_files = construct_paths(&source_files, path);
    for (input_file, output_file) in source_files.into_iter().zip(output_files) {
        let tokenizer = Tokenizer::new();
        match tokenizer.lex_file(input_file) {
            Ok(q) => {
                if let Some(TokenType::Error(msg)) = q.back().map(|t| t.token_type()) {
                    err_vec.push(LexerError::ErrorToken(msg.to_string()))
                }
                token_queues.push((q, output_file));
            }
            //Only send file io errors upstream otherwise lexer errors should print error to lexed file
            //or maybe print to terminal if --paser or --compile selected?
            Err(e) => err_vec.push(e),
        };
    }
    if !err_vec.is_empty() {
        return Err(err_vec);
    }
    Ok(token_queues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    fn create_path(file_name: &str) -> String {
        let mut prefix = PathBuf::from("eta_programs/lexer_files/");
        prefix.push(file_name);
        prefix.to_str().expect("valid utf-8").to_string()
    }

    #[test]
    fn test_verification_pass() -> Result<(), LexerError> {
        let source_files = vec![
            create_path("lex_test_1.eta"),
            create_path("lex_test_2.eta"),
            create_path("lex_interface.eti"),
        ];
        verify_files(&source_files)
    }

    #[test]
    fn test_verfication_multiple_fail() {
        let source_files = vec![
            create_path("lex_test_1.eta"),
            create_path("lex_interface.eta"),
            create_path("fail_verification.txt"),
            create_path("fail_verification_2.txt"),
        ];
        assert!(verify_files(&source_files).is_err())
    }

    #[test]
    fn test_single_failure() {
        let source_files = vec![create_path("fail_verification.txt")];
        assert!(verify_files(&source_files).is_err())
    }

    #[test]
    fn test_output_conversion_no_path() {
        let source_files = vec![
            create_path("lex_test_1.eta"),
            create_path("lex_test_2.eta"),
            create_path("lex_interface.eti"),
        ];
        dbg!(&source_files);
        let paths = construct_paths(&source_files, PathBuf::from(""));
        for path in paths {
            assert!(path.extension() == Some(&OsStr::new("lexed")))
        }
    }

    #[test]
    fn test_output_conversion_with_path() {
        let source_files = vec![
            create_path("lex_test_1.eta"),
            create_path("lex_test_2.eta"),
            create_path("lex_interface.eti"),
        ];
        let targets = vec![
            PathBuf::from(
                "/home/togal/Eta-Compiler/output/eta_programs/lexer_files/lex_test_1.lexed",
            ),
            PathBuf::from(
                "/home/togal/Eta-Compiler/output/eta_programs/lexer_files/lex_test_2.lexed",
            ),
            PathBuf::from(
                "/home/togal/Eta-Compiler/output/eta_programs/lexer_files/lex_interface.lexed",
            ),
        ];
        let paths = construct_paths(
            &source_files,
            PathBuf::from("/home/togal/Eta-Compiler/output"),
        );
        for (path, target) in paths.into_iter().zip(targets) {
            assert_eq!(path, target);
        }
    }
}
