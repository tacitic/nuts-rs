use failure::{Backtrace, Context, Fail};
use std::fmt;

/// The specific kind of error that can occur.
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    /// A serialization / deserialization error
    #[fail(display = "Serialization error: {}", _0)]
    SerdeError(String),
    /// An unknown error.
    #[fail(display = "An unknown error occurred.")]
    Unknown,
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    #[fail(display = "Non exhaustive")]
    __Nonexhaustive,
}

/// An error that can occur while interacting with Octokit
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    /// Return the kind of this error.
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}
