#[derive(Debug)]
pub enum LexerErrorType {
    UnexpectedEndOfString,
    UnknownToken
}

#[derive(Debug)]
pub enum ErrorType {
    LexerError(LexerErrorType)
}

#[derive(Debug)]
pub struct Error {
    pub pos: usize,
    pub width: usize,
    pub error_type: ErrorType
}

impl Error {
    pub fn new(pos: usize, width: usize, error_type: ErrorType) -> Self {
        Self { pos, width, error_type }
    }
}