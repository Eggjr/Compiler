#[derive(Debug)]
pub enum LexerError {
    InvalidFilesError(Vec<String>),
    IOError(String, std::io::Error),
    ErrorToken(String),
}
