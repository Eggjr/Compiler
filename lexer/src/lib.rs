//! # Lexer
//!
//! `Lexer` is a crate for lexing the chuda programming language
use std::collections::VecDeque;

mod lexer_error;
mod token;
mod token_scanner;
mod token_type;
mod tokenizer;

pub use lexer_error::LexerError;
pub use token::Token;
pub use token_type::TokenType;
pub use tokenizer::Tokenizer;

/// Lexes `source_files` and returns a token representation of the contents with the corresponding
/// `path`/file_name.lexed output file
///
/// # Examples
///```
/// use std::collections::VecDeque;
/// use lexer::TokenType;
/// use lexer::Token;
/// use pretty_assertions::assert_eq;
/// use std::fs;
///
/// let mut tokens : VecDeque<Token> = VecDeque::new();
/// tokens.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
/// tokens.push_back(Token::new(1, 6, TokenType::Colon));
/// tokens.push_back(Token::new(1, 7, TokenType::Int));
/// tokens.push_back(Token::new(1, 10, TokenType::Assign));
/// tokens.push_back(Token::new(1, 11, TokenType::Integer(10)));
/// let vec_tokens = vec![tokens];
///
/// let (tokens, errs) = lexer::lex_files(&vec![fs::read_to_string("../chuda_programs/lexer_files/lex_test_3.chuda".to_string()).expect("Read Failure")], );
/// assert_eq!(tokens, vec_tokens);
/// assert!(errs.is_none());
/// ```
///
/// # Errors
///
/// returns `Vec<LexerError>` upon failure to verify files, or upon termination if any files had
/// an error token written to them, dchudailing where the errors can be viewed and giving a brief
/// description of the errors
pub fn lex_files(source_texts: &[String]) -> (Vec<VecDeque<Token>>, Option<Vec<LexerError>>) {
    let mut err_vec = vec![];
    let mut token_queues: Vec<VecDeque<Token>> = vec![];
    for source_text in source_texts {
        let tokenizer = Tokenizer::new();
        let q = tokenizer.lex_file(source_text);
        if let Some(TokenType::Error(msg)) = q.back().map(|t| t.token_type()) {
            err_vec.push(LexerError::ErrorToken(msg.to_string()))
        }
        token_queues.push(q);
    }
    if !err_vec.is_empty() {
        return (token_queues, Some(err_vec));
    }
    (token_queues, None)
}
