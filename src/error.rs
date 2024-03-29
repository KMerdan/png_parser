use std::io;

#[derive(Debug)]
pub enum PngError {
    Io(io::Error),
    InvalidSignature,
    InvalidChunkSequence,
    InvalidData,
    ParseError(String),
}

impl From<io::Error> for PngError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::InvalidData {
            PngError::InvalidSignature
        } else {
            PngError::Io(err)
        }
    }
}

impl From<&str> for PngError {
    fn from(err: &str) -> Self {
        PngError::ParseError(err.to_string())
    }
}

impl From<String> for PngError {
    fn from(err: String) -> Self {
        PngError::ParseError(err)
    }
}

impl std::fmt::Display for PngError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PngError::Io(err) => write!(f, "IO Error: {}", err),
            PngError::InvalidSignature => write!(f, "Invalid PNG Signature"),
            PngError::ParseError(err) => write!(f, "Parse Error: {}", err),
            PngError::InvalidChunkSequence => write!(f, "Invalid Chunk Sequence"),
            PngError::InvalidData => {
                write!(f, "Invalid Data: Insufficient bytes for header information")
            }
        }
    }
}

impl std::error::Error for PngError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PngError::Io(err) => Some(err),
            _ => None,
        }
    }
}
