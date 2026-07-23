use std::fmt;
#[derive(Debug)]
pub enum LexerError {
    ErrorToken(String),
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rep = match self {
            LexerError::ErrorToken(e) => e.to_string(),
        };
        write!(f, "{}", rep)
    }
}
