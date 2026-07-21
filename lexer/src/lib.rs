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

/// Writes `tokens` to specified `writer` for diagnostic purposes
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
/// lexer::write_tokens(&mut tokens, &mut buffer).expect("Writing failed");
/// assert_eq!(String::from_utf8(buffer).expect("This is valid utf-8"), "1:1 id Pizza\n1:6 :\n1:7 int\n1:10 =\n1:11 integer 10\n");
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
/// use std::collections::VecDeque;
/// use lexer::TokenType;
/// use lexer::Token;
///
/// let mut tokens : VecDeque<Token> = VecDeque::new();
/// tokens.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
/// tokens.push_back(Token::new(1, 6, TokenType::Colon));
/// tokens.push_back(Token::new(1, 7, TokenType::Int));
/// tokens.push_back(Token::new(1, 10, TokenType::Assign));
/// tokens.push_back(Token::new(1, 11, TokenType::Integer(10)));
/// let vec_tokens = vec![tokens];
///
/// let tokens = match lexer::lex_files(&vec!["../chuda_programs/lexer_files/lex_test_3.chuda".to_string()]){
///     Ok(v) => v,
///     Err(e) => {
///         panic!("Lexing Failed")
///     },
/// };
/// assert_eq!(tokens, vec_tokens);
/// ```
///
/// # Errors
///
/// returns `Vec<LexerError>` upon failure to verify files, or upon termination if any files had
/// an error token written to them, dchudailing where the errors can be viewed and giving a brief
/// description of the errors
pub fn lex_files(source_files: &[String]) -> Result<Vec<VecDeque<Token>>, Vec<LexerError>> {
    let mut err_vec = vec![];
    let mut token_queues: Vec<VecDeque<Token>> = vec![];
    for input_file in source_files {
        let tokenizer = Tokenizer::new();
        match tokenizer.lex_file(input_file) {
            Ok(q) => {
                if let Some(TokenType::Error(msg)) = q.back().map(|t| t.token_type()) {
                    err_vec.push(LexerError::ErrorToken(msg.to_string()))
                }
                token_queues.push(q);
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
