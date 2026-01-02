mod base64;
#[cfg(feature = "auth")]
mod keyring;
#[cfg(unix)]
mod nix;
pub mod reqwest;
mod serde;
mod std_io;
mod utf8;

// Re-export the Error type from human_errors
pub type Error = human_errors::Error;

// A simple error type for wrapping string messages as errors
#[derive(Debug)]
pub struct StringError(String);

impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

impl StringError {
    pub fn new(message: impl Into<String>) -> Self {
        StringError(message.into())
    }
}
