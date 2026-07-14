#[derive(Debug)]
pub enum LexerError {
    IOReadError(String, std::io::Error),
    IOWriteError(std::io::Error),
    ErrorToken(String),
}
