use std::{error, fmt};

mod base64;
pub mod hyper;
mod keyring;
#[cfg(unix)]
mod nix;
mod serde;
mod std_io;
mod utf8;

#[derive(Debug)]
pub enum Error {
    UserError(
        String,
        String,
        Option<Box<Error>>,
        Option<Box<dyn error::Error + Send + Sync>>,
    ),
    SystemError(
        String,
        String,
        Option<Box<Error>>,
        Option<Box<dyn error::Error + Send + Sync>>,
    ),
}

pub fn user(description: &str, advice: &str) -> Error {
    Error::UserError(description.to_string(), advice.to_string(), None, None)
}

pub fn user_with_cause(description: &str, advice: &str, cause: Error) -> Error {
    Error::UserError(
        description.to_string(),
        advice.to_string(),
        Some(Box::from(cause)),
        None,
    )
}

pub fn user_with_internal<T>(description: &str, advice: &str, internal: T) -> Error
where
    T: Into<Box<dyn error::Error + Send + Sync>>,
{
    Error::UserError(
        description.to_string(),
        advice.to_string(),
        None,
        Some(internal.into()),
    )
}

pub fn system(description: &str, advice: &str) -> Error {
    Error::SystemError(description.to_string(), advice.to_string(), None, None)
}

pub fn system_with_cause(description: &str, advice: &str, cause: Error) -> Error {
    Error::SystemError(
        description.to_string(),
        advice.to_string(),
        Some(Box::from(cause)),
        None,
    )
}

pub fn system_with_internal<T>(description: &str, advice: &str, internal: T) -> Error
where
    T: Into<Box<dyn error::Error + Send + Sync>>,
{
    Error::SystemError(
        description.to_string(),
        advice.to_string(),
        None,
        Some(internal.into()),
    )
}

impl Error {
    pub fn description(&self) -> String {
        match self {
            Error::UserError(description, ..) | Error::SystemError(description, ..) => {
                description.clone()
            }
        }
    }

    pub fn message(&self) -> String {
        let (description, internal) = match self {
            Error::UserError(description, _, _, internal)
            | Error::SystemError(description, _, _, internal) => (description, internal),
        };

        let hero_message = match self {
            Error::UserError(_, _, _, _) => {
                format!("Oh no! {}", description)
            }
            Error::SystemError(_, _, _, _) => {
                format!("Whoops! {} (This isn't your fault)", description)
            }
        };

        match self.caused_by() {
            Some(cause) => {
                format!(
                    "{}\n\nThis was caused by:\n{}\n\nTo try and fix this, you can:\n{}",
                    hero_message,
                    cause,
                    self.advice()
                )
            }
            None => {
                format!(
                    "{}\n\nTo try and fix this, you can:\n{}",
                    hero_message,
                    self.advice()
                )
            }
        }
    }

    fn caused_by(&self) -> Option<String> {
        match self {
            Error::UserError(.., Some(cause), _) | Error::SystemError(.., Some(cause), _) => {
                match cause.caused_by() {
                    Some(child_cause) => {
                        Some(format!(" - {}\n{}", cause.description(), child_cause))
                    }
                    None => Some(format!(" - {}", cause.description())),
                }
            }
            Error::UserError(.., Some(internal)) | Error::SystemError(.., Some(internal)) => {
                Some(format!(" - {}", internal))
            }
            _ => None,
        }
    }

    fn advice(&self) -> String {
        let (advice, cause) = match self {
            Error::UserError(_, advice, cause, _) | Error::SystemError(_, advice, cause, _) => {
                (advice, cause)
            }
        };

        match cause {
            // We bias towards the most specific advice first (i.e. the lowest-level error) because that's most likely to be correct.
            Some(cause) => format!("{}\n - {}", cause.advice(), advice),
            None => format!(" - {}", advice),
        }
    }

    pub fn is_system(&self) -> bool {
        match self {
            Error::SystemError(..) => true,
            _ => false,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::UserError(.., Some(ref err)) | Error::SystemError(.., Some(ref err)) => {
                err.source()
            }
            _ => None,
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
    message: String,
}

impl From<&str> for BasicInternalError {
    fn from(s: &str) -> Self {
        Self {
            message: s.to_string(),
        }
    }
}

impl std::error::Error for BasicInternalError {}

impl fmt::Display for BasicInternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
