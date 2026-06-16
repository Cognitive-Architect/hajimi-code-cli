#[derive(Debug)]
pub enum Error {
    Generic(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Generic(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for Error {}
