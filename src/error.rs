use std::io;

#[derive(Debug)]
pub enum PngError {
    Io(io::Error),
    InvalidSignature,
    ParseError(String),
}

impl From<io::Error> for PngError {
    fn from(err: io::Error) -> Self {
        PngError::Io(err)
    }
}
