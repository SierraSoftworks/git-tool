use std::fmt;

mod std_io;
mod hyper;
mod utf8;
mod base64;
mod serde;

#[derive(Debug, Clone)]
pub struct Error {
    description: String,
    advice: String,
    internal: String,
    is_system: bool,
}

/// Creates a new `Error` with the provided description, advice and internal error,
/// indicating whether it is a system-level error or not.
pub fn new<T>(description: &str, advice: &str, internal: T, is_system: bool) -> Error
where T : fmt::Debug {
    Error {
        description: String::from(description),
        advice: String::from(advice),
        internal: format!("{:#?}", internal),
        is_system,
    }
}

pub fn user(description: &str, advice: &str) -> Error {
    new(description, advice, "", false)
}

pub fn user_with_internal<T>(description: &str, advice: &str, internal: T) -> Error
where T : fmt::Debug {
    new(description, advice, internal, false)
}

pub fn system(description: &str, advice: &str) -> Error {
    new(description, advice, "", true)
}

pub fn system_with_internal<T>(description: &str, advice: &str, internal: T) -> Error
where T : fmt::Debug {
    new(description, advice, internal, true)
}

impl Error {
    pub fn message(&self) -> String {
        if self.internal.is_empty() || self.internal == "\"\"" {
            if self.is_system {
                format!("Whoops! {} (This isn't your fault)\nAdvice: {}", self.description, self.advice)
            } else {
                format!("Oh no! {}\nAdvice: {}", self.description, self.advice)
            }
        } else {
            if self.is_system {
                format!("Whoops! {} (This isn't your fault)\nAdvice: {}\n\n{}", self.description, self.advice, self.internal)
            } else {
                format!("Oh no! {}\nAdvice: {}\n\n{}", self.description, self.advice, self.internal)
            }
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}
