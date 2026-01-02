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
