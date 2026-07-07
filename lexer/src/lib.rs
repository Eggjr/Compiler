mod lexer_error;
mod token;
mod token_scanner;
mod token_type;
mod tokenizer;

pub use lexer_error::LexerError;
pub use token::Token;
pub use token_type::TokenType;
pub use tokenizer::Tokenizer;

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

fn verify_files(source_files: &Vec<String>) -> Result<(), LexerError> {
    let invalids: Vec<String> = source_files
        .iter()
        .filter(|file| !file.ends_with(".eta") && !file.ends_with(".eti"))
        .cloned()
        .collect();
    if !invalids.is_empty() {
        return Err(LexerError::InvalidFilesError(invalids));
    }
    return Ok(());
}

fn construct_paths(source_files: &Vec<String>, path: PathBuf) -> Vec<PathBuf> {
    return source_files
        .iter()
        .map(|file_name| (path.join(file_name)).with_extension("lexed"))
        .collect();
}

pub fn write_to_file(tokens: &mut VecDeque<Token>, output_file: PathBuf) -> Result<(), LexerError> {
    let output_text = tokens
        .drain(..)
        .map(|token| token.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    match fs::write(output_file.as_path(), output_text) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(LexerError::IOError(
                format!(
                    "Trouble writing to file: {:?} \nMaybe it is an invalid name?",
                    output_file.as_os_str()
                ),
                e,
            ));
        }
    }
}

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
    for (input_file, output_file) in source_files.into_iter().zip(output_files.into_iter()) {
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

    #[test]
    fn test_verification_pass() {}
}
