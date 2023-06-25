use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Ureq(Box<ureq::Error>),
    Io(std::io::Error),
    Descriptive(String),
    Abort,
}

impl From<Box<ureq::Error>> for Error {
    fn from(error: Box<ureq::Error>) -> Self {
        Self::Ureq(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ureq(error) => write!(f, "Network error:\n{:#?}", error),
            Self::Io(error) => write!(f, "IO error:\n{:#?}", error),
            Self::Descriptive(message) => write!(f, "{}", message),
            Self::Abort => write!(f, "Aborted."),
        }
    }
}
