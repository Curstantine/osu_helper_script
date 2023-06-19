use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Ureq { error: Box<ureq::Error> },
    Io { error: std::io::Error },
}

impl From<Box<ureq::Error>> for Error {
    fn from(error: Box<ureq::Error>) -> Self {
        Self::Ureq { error }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io { error }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ureq { error } => write!(f, "Network error: {:#?}", error),
            Self::Io { error } => write!(f, "IO error: {:#?}", error),
        }
    }
}
