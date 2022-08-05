use std::fmt;

#[derive(Debug)]
pub enum ErrorEnum {
    Http(http::Error),
    IO(std::io::Error),
    Other(Box<dyn std::error::Error + Send>),
}

impl std::error::Error for ErrorEnum {}

impl std::fmt::Display for ErrorEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(err) => err.fmt(f),
            Self::IO(err) => err.fmt(f),
            Self::Other(err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for ErrorEnum {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<http::Error> for ErrorEnum {
    fn from(err: http::Error) -> Self {
        Self::Http(err)
    }
}

impl From<Box<dyn std::error::Error + Send>> for ErrorEnum {
    fn from(err: Box<dyn std::error::Error + Send>) -> Self {
        Self::Other(err)
    }
}
