pub use human_errors::detailed_message;

mod base64;
mod keyring;
#[cfg(unix)]
mod nix;
pub mod reqwest;
mod serde;
mod std_io;
mod utf8;

human_errors::error_shim!(Error);
