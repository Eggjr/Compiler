use crate::token_type::TokenType;
use std::fmt;
#[derive(Debug, Clone)]
pub struct Token {
    line: usize,
    column: usize,
    token_type: TokenType,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} {}",
            self.line,
            self.column,
            self.token_type.to_string()
        )
    }
}

impl Token {
    pub fn new(line: usize, column: usize, token_type: TokenType) -> Token {
        return Token {
            line,
            column,
            token_type,
        };
    }

    pub fn token_type(&self) -> &TokenType {
        return &self.token_type;
    }
}
