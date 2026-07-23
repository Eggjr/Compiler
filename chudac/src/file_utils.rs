use std::collections::VecDeque;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

use crate::file_utils::CompilerError::{IORead, IOWrite, InvalidFiles};

#[derive(Debug)]
pub enum CompilerError {
    InvalidFiles(Vec<String>),
    IORead(String, std::io::Error),
    IOWrite(std::io::Error),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            InvalidFiles(invalids) => format!("Invalid files given: {:?}", invalids),
            IORead(file, e) => format!("Tried reading {} but failed with: {}", file, e),
            IOWrite(e) => format!("Tried writing output but failed with: {}", e),
        };
        write!(f, "{}", msg)
    }
}

/// Creates a file and all its parent directories if they do not already exist
pub fn safe_create_file(path: &PathBuf) -> io::Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(path)
}

/// Checks if `source_files` only contains file names with *.chuda* or *.chudi* endings
///
/// # Errors
///
/// returns `CompilerError::InvalidFilesError(invalids)` where `invalids` is a list of files with non *.chuda* or *.chudi* endings
fn verify_files(source_files: &[String]) -> Result<(), CompilerError> {
    let invalids: Vec<String> = source_files
        .iter()
        .filter(|file| !file.ends_with(".chuda") && !file.ends_with(".chudi"))
        .cloned()
        .collect();
    if !invalids.is_empty() {
        return Err(InvalidFiles(invalids));
    }
    Ok(())
}

/// Constructs output `.ending` files with the specified prefix `path` for each file name in `source_files`
///
/// # Requires
///
/// All files in `source_files` end with `.chuda* or *.chudi*
fn construct_paths(source_files: &[String], path: PathBuf, ending: &str) -> Vec<PathBuf> {
    source_files
        .iter()
        .map(|file_name| path.join(file_name).with_extension(ending))
        .collect()
}

/// Verfies `sources_files` are *.chuda* or *.chudi* files, and constructs output files with extension `ending` and prefix `path`
///
/// # Errors
///
/// Returns `CompilerError::InvalidFilesError(invalid_files)` if `source_files` were not *.chuda* or *.chudi* files
pub fn verify_and_construct(
    source_files: &[String],
    path: PathBuf,
    ending: &str,
) -> Result<Vec<PathBuf>, CompilerError> {
    verify_files(source_files)?;
    Ok(construct_paths(source_files, path, ending))
}

/// reads specified `source_files` found at `source_path` to strings for tokenization
pub fn read_source_files(
    source_files: &[String],
    source_path: PathBuf,
) -> Result<Vec<String>, Vec<CompilerError>> {
    let mut err_vec = Vec::new();
    let mut source_texts = Vec::new();
    for file in source_files {
        match fs::read_to_string(source_path.join(file)) {
            Ok(s) => source_texts.push(s),
            Err(e) => err_vec.push(IORead(file.clone(), e)),
        }
    }
    if !err_vec.is_empty() {
        return Err(err_vec);
    }
    Ok(source_texts)
}

/// Writes `fmt_deque` to specified `writer` for diagnostic purposes
///
/// # Examples
///
/// ```
/// use lexer::Token;
/// use lexer::TokenType;
/// use std::collections::VecDeque;
///
/// let mut buffer = vec![];
/// let mut tokens : VecDeque<Token> = VecDeque::new();
/// tokens.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
/// tokens.push_back(Token::new(1, 6, TokenType::Colon));
/// tokens.push_back(Token::new(1, 7, TokenType::Int));
/// tokens.push_back(Token::new(1, 10, TokenType::Assign));
/// tokens.push_back(Token::new(1, 11, TokenType::Integer(10)));
/// file_utils::write_deque(&mut tokens, &mut buffer).expect("Writing failed");
/// assert_eq!(String::from_utf8(buffer).expect("This is valid utf-8"), "1:1 id Pizza\n1:6 :\n1:7 int\n1:10 =\n1:11 integer 10\n");
/// ```
///
/// # Errors
///
/// returns `CompilerError::IOWriteError` if it failed to write to `writer`
pub fn write_deque<W: std::io::Write, F: std::fmt::Display>(
    fmt_deque: &mut VecDeque<F>,
    writer: &mut W,
) -> Result<(), CompilerError> {
    while let Some(item) = fmt_deque.pop_front() {
        if let Err(e) = writeln!(writer, "{}", item) {
            return Err(IOWrite(e));
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    fn create_path(file_name: &str) -> String {
        let mut prefix = PathBuf::from("chuda_programs/lexer_files/");
        prefix.push(file_name);
        prefix.to_str().expect("valid utf-8").to_string()
    }

    #[test]
    fn test_verification_pass() -> Result<(), CompilerError> {
        let source_files = vec![
            create_path("lex_test_1.chuda"),
            create_path("lex_test_2.chuda"),
            create_path("lex_interface.chudi"),
        ];
        verify_files(&source_files)
    }

    #[test]
    fn test_verfication_multiple_fail() {
        let source_files = vec![
            create_path("lex_test_1.chuda"),
            create_path("lex_interface.chuda"),
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
            create_path("lex_test_1.chuda"),
            create_path("lex_test_2.chuda"),
            create_path("lex_interface.chudi"),
        ];
        dbg!(&source_files);
        let paths = construct_paths(&source_files, PathBuf::from(""), "lexed");
        for path in paths {
            assert!(path.extension() == Some(&OsStr::new("lexed")))
        }
    }

    #[test]
    fn test_output_conversion_with_path() {
        let source_files = vec![
            create_path("lex_test_1.chuda"),
            create_path("lex_test_2.chuda"),
            create_path("lex_interface.chudi"),
        ];
        let targets = vec![
            PathBuf::from(
                "/home/togal/chuda-Compiler/output/chuda_programs/lexer_files/lex_test_1.lexed",
            ),
            PathBuf::from(
                "/home/togal/chuda-Compiler/output/chuda_programs/lexer_files/lex_test_2.lexed",
            ),
            PathBuf::from(
                "/home/togal/chuda-Compiler/output/chuda_programs/lexer_files/lex_interface.lexed",
            ),
        ];
        let paths = construct_paths(
            &source_files,
            PathBuf::from("/home/togal/chuda-Compiler/output"),
            "lexed",
        );
        for (path, target) in paths.into_iter().zip(targets) {
            assert_eq!(path, target);
        }
    }
}
