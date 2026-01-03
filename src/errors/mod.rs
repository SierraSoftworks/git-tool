mod base64;
#[cfg(feature = "auth")]
mod keyring;
#[cfg(unix)]
mod nix;
pub mod reqwest;
mod serde;
mod std_io;
mod utf8;

pub trait HumanErrorExt {
    fn to_human_error(self) -> human_errors::Error;
}

pub trait HumanErrorResultExt<T> {
    fn to_human_error(self) -> Result<T, human_errors::Error>;
}

impl<T, E: HumanErrorExt> HumanErrorResultExt<T> for Result<T, E> {
    fn to_human_error(self) -> Result<T, human_errors::Error> {
        self.map_err(|e| e.to_human_error())
    }
}
