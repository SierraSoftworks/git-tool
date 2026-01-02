mod base64;
#[cfg(feature = "auth")]
mod keyring;
#[cfg(unix)]
mod nix;
pub mod reqwest;
mod serde;
mod std_io;
mod utf8;

#[derive(Debug)]
pub struct Error(human_errors::Error);

impl Error {
    pub fn is_system(&self) -> bool {
        self.0.is_system()
    }

    pub fn description(&self) -> &str {
        self.0.description()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl From<human_errors::Error> for Error {
    fn from(err: human_errors::Error) -> Self {
        Error(err)
    }
}

#[derive(Debug)]
pub struct DetailedMessage(String);

impl std::fmt::Display for DetailedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DetailedMessage {}

pub fn user(message: impl Into<std::borrow::Cow<'static, str>> + 'static, advice: &'static str) -> Error {
    Error(human_errors::user(message, vec![advice.to_string()]))
}

pub fn user_with_internal<E: std::error::Error + Send + Sync + 'static>(
    message: impl Into<std::borrow::Cow<'static, str>> + 'static,
    advice: &'static str,
    cause: E,
) -> Error {
    Error(human_errors::wrap_user(cause, message, vec![advice.to_string()]))
}

pub fn user_with_cause<E: std::error::Error + Send + Sync + 'static>(
    message: impl Into<std::borrow::Cow<'static, str>> + 'static,
    advice: &'static str,
    cause: E,
) -> Error {
    Error(human_errors::wrap_user(cause, message, vec![advice.to_string()]))
}

pub fn system_with_internal<E: std::error::Error + Send + Sync + 'static>(
    message: impl Into<std::borrow::Cow<'static, str>> + 'static,
    advice: &'static str,
    cause: E,
) -> Error {
    Error(human_errors::wrap_system(cause, message, vec![advice.to_string()]))
}

pub fn system(message: impl Into<std::borrow::Cow<'static, str>> + 'static, advice: &'static str) -> Error {
    Error(human_errors::system(message, vec![advice.to_string()]))
}

pub fn detailed_message(message: &str) -> DetailedMessage {
    DetailedMessage(message.to_string())
}
