use crate::lexer_error::LexerError;
use crate::token::Token;
use crate::token_scanner::TokenScanner;
use crate::token_type::TokenType;
use std::collections::VecDeque;
use std::fs;

#[derive(Debug, Clone)]
pub struct Tokenizer {
    tokens: VecDeque<Token>,
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        Tokenizer {
            tokens: VecDeque::new(),
        }
    }

    pub fn lex_file(mut self, input_file: String) -> Result<VecDeque<Token>, LexerError> {
        let input_text = match fs::read_to_string(&input_file) {
            Ok(text) => text,
            Err(e) => return Err(LexerError::IOError(input_file, e)),
        };
        let mut scanner: TokenScanner = TokenScanner::build(&input_text);
        while let Some(token) = scanner.next_token() {
            if let TokenType::Error(_) = token.token_type() {
                self.tokens.push_back(token);
                break;
            }
            self.tokens.push_back(token);
        }
        Ok(self.tokens)
    }
}
