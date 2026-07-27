use crate::token::Token;
use crate::token_scanner::TokenScanner;
use crate::token_type::TokenType;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct Tokenizer {
    tokens: VecDeque<Token>,
}

impl Tokenizer {
    pub fn new() -> Tokenizer {
        Tokenizer {
            tokens: VecDeque::new(),
        }
    }

    /// lexes `input_file` and returns a `VecDeque<Token>` representation of the file
    ///
    /// # Errors
    ///
    /// return `LexerError::IOReadError` if it could not read `input_file`
    pub fn lex_file(mut self, source_text: &str) -> VecDeque<Token> {
        let mut scanner: TokenScanner = TokenScanner::new(source_text);
        loop {
            let token = scanner.next_token();
            match token.token_type(){
                TokenType::Error(_) | TokenType::Eof =>{
                    self.tokens.push_back(token);
                    break;
                },
                _ => self.tokens.push_back(token)
            }
        }
        self.tokens
    }
}
