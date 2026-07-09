#[derive(Debug)]
pub enum LexerError {
    InvalidFilesError(Vec<String>),
    IOReadError(String, std::io::Error),
    IOWriteError(std::io::Error),
    ErrorToken(String),
    CanonicalizationFailure(std::io::Error),
    PathCreationFailure(std::io::Error),
}
