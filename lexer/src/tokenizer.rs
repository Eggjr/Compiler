use crate::lexer_error::LexerError;
use crate::token::Token;
use crate::token_scanner::TokenScanner;
use crate::token_type::TokenType;
use std::collections::VecDeque;
use std::fs;

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

    /// lexes `input_file` and returns a VecDeque<Token> representation of the file
    ///
    /// # Examples
    /// ```
    /// use lexer::TokenType;
    /// use lexer::Token;
    /// use lexer::Tokenizer;
    /// use std::collections::VecDeque;
    ///
    /// let tokenizer = Tokenizer::new();
    /// let tokens = tokenizer.lex_file("../eta_programs/lexer_files/lex_test_3.eta".to_string()).expect("Lexing Failure");
    /// let mut targets : VecDeque<Token> = VecDeque::new();
    /// targets.push_back(Token::new(1, 1, TokenType::Identifier("Pizza".to_string())));
    /// targets.push_back(Token::new(1, 6, TokenType::Colon));
    /// targets.push_back(Token::new(1, 7, TokenType::Int));
    /// targets.push_back(Token::new(1, 10, TokenType::Assign));
    /// targets.push_back(Token::new(1, 11, TokenType::Integer(10)));
    /// assert_eq!(tokens, targets)
    /// ```
    ///
    /// # Errors
    ///
    /// return LexerError::IOReadError if it could not read `input_file`
    pub fn lex_file(mut self, input_file: String) -> Result<VecDeque<Token>, LexerError> {
        let input_text = match fs::read_to_string(&input_file) {
            Ok(text) => text,
            Err(e) => return Err(LexerError::IOReadError(input_file, e)),
        };
        let mut scanner: TokenScanner = TokenScanner::new(&input_text);
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
