use core::fmt;

pub mod excellon;
pub mod gerber;

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    Gerber(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "IO error: {}", e),
            ParseError::Gerber(msg) => write!(f, "Gerber parse error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(value: std::io::Error) -> Self {
        ParseError::Io(value)
    }
}
