extern crate image;
extern crate syntect;
extern crate tera;
extern crate toml;

use std::convert::Into;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    Msg(String),
    Tera(tera::Error),
    Io(::std::io::Error),
    Toml(toml::de::Error),
    Image(image::ImageError),
    Syntect(syntect::LoadingError),
}

/// The Error type
#[derive(Debug)]
pub struct Error {
    /// Kind of error
    pub kind: ErrorKind,
    pub source: Option<Box<dyn StdError>>,
}
unsafe impl Sync for Error {}
unsafe impl Send for Error {}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        let mut source = self.source.as_ref().map(|c| &**c);
        if source.is_none() {
            match self.kind {
                ErrorKind::Tera(ref err) => source = err.source(),
                _ => (),
            };
        }

        source
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Msg(ref message) => write!(f, "{}", message),
            ErrorKind::Tera(ref e) => write!(f, "{}", e),
            ErrorKind::Io(ref e) => write!(f, "{}", e),
            ErrorKind::Toml(ref e) => write!(f, "{}", e),
            ErrorKind::Image(ref e) => write!(f, "{}", e),
            ErrorKind::Syntect(ref e) => write!(f, "{}", e),
        }
    }
}

impl Error {
    /// Creates generic error
    pub fn msg(value: impl ToString) -> Self {
        Self { kind: ErrorKind::Msg(value.to_string()), source: None }
    }

    /// Creates generic error with a cause
    pub fn chain(value: impl ToString, source: impl Into<Box<dyn StdError>>) -> Self {
        Self { kind: ErrorKind::Msg(value.to_string()), source: Some(source.into()) }
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Self::msg(e)
    }
}
impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::msg(e)
    }
}
impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self { kind: ErrorKind::Toml(e), source: None }
    }
}
impl From<syntect::LoadingError> for Error {
    fn from(e: syntect::LoadingError) -> Self {
        Self { kind: ErrorKind::Syntect(e), source: None }
    }
}
impl From<tera::Error> for Error {
    fn from(e: tera::Error) -> Self {
        Self { kind: ErrorKind::Tera(e), source: None }
    }
}
impl From<::std::io::Error> for Error {
    fn from(e: ::std::io::Error) -> Self {
        Self { kind: ErrorKind::Io(e), source: None }
    }
}
impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Self { kind: ErrorKind::Image(e), source: None }
    }
}
/// Convenient wrapper around std::Result.
pub type Result<T> = ::std::result::Result<T, Error>;

// So we can use bail! in all other crates
#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err($e.into());
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err(format!($fmt, $($arg)+).into());
    };
}
