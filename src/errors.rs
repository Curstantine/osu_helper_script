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

impl From<inquire::InquireError> for Error {
    fn from(value: inquire::InquireError) -> Self {
        use inquire::InquireError;

        match value {
            InquireError::OperationInterrupted | InquireError::OperationCanceled => Error::Abort,
            InquireError::IO(io_error) => Error::Io(io_error),
            _ => panic!("Unhandled error: {:#?}", value),
        }
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

/// Helper function to simplify the error handling for IO operations that should ignore [std::io::ErrorKind::NotFound].
pub fn ignore_io_not_found(res: std::io::Result<()>, done_msg: String, not_found_msg: String) -> Result<()> {
    match res {
        Ok(_) => {
            println!("\r{}", done_msg);
            Ok(())
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                println!("\r{}", not_found_msg);
                Ok(())
            } else {
                Err(Error::from(e))
            }
        }
    }
}
