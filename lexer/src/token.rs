use crate::token_type::TokenType;
use std::fmt;
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    line: usize,
    column: usize,
    token_type: TokenType,
}

/// String representation of token to be printed to file as described by spec
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} {}", self.line, self.column, self.token_type)
    }
}

impl Token {
    /// creates a new token
    pub fn new(line: usize, column: usize, token_type: TokenType) -> Token {
        Token {
            line,
            column,
            token_type,
        }
    }

    /// allows for view into private field token_type
    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }
}
