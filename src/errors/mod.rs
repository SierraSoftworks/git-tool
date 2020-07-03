use std::{fmt, error};

mod std_io;
pub mod hyper;
mod utf8;
mod base64;
mod serde;
mod keyring;

#[derive(Debug)]
pub enum Error {
    UserError(String, String, Option<Box<dyn error::Error + Send + Sync>>),
    SystemError(String, String, Option<Box<dyn error::Error + Send + Sync>>)
}

pub fn user(description: &str, advice: &str) -> Error {
    Error::UserError(description.to_string(), advice.to_string(), None)
}

pub fn user_with_internal<T>(description: &str, advice: &str, internal: T) -> Error
where T : Into<Box<dyn error::Error + Send + Sync>> {
    Error::UserError(description.to_string(), advice.to_string(), Some(internal.into()))
}

pub fn system(description: &str, advice: &str) -> Error {
    Error::SystemError(description.to_string(), advice.to_string(), None)
}

pub fn system_with_internal<T>(description: &str, advice: &str, internal: T) -> Error
where T : Into<Box<dyn error::Error + Send + Sync>> {
    Error::SystemError(description.to_string(), advice.to_string(), Some(internal.into()))
}

impl Error {
    pub fn message(&self) -> String {
        match self {
            Error::UserError(description, advice, None) => {
                format!("Oh no! {}\nAdvice: {}", description, advice)
            },
            Error::UserError(description, advice, Some(internal)) => {
                format!("Oh no! {}\nAdvice: {}\n\n{}", description, advice, internal)
            },
            Error::SystemError(description, advice, None) => {
                format!("Whoops! {} (This isn't your fault)\nAdvice: {}", description, advice)
            },
            Error::SystemError(description, advice, Some(internal)) => {
                format!("Whoops! {} (This isn't your fault)\nAdvice: {}\n\n{}", description, advice, internal)
            }
        }
    }

    pub fn is_system(&self) -> bool {
        match self {
            Error::SystemError(..) => true,
            _ => false
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::UserError(.., Some(ref err)) | Error::SystemError(_, _, Some(ref err)) => {
                err.source()
            },
            _ => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

pub fn detailed_message(message: &str) -> BasicInternalError {
    message.into()
}

#[derive(Debug)]
pub struct BasicInternalError {
    message: String
}

impl From<&str> for BasicInternalError {
    fn from(s: &str) -> Self {
        Self {
            message: s.to_string()
        }
    }
}

impl std::error::Error for BasicInternalError {

}

impl fmt::Display for BasicInternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}