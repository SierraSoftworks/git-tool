use super::{Error, system_with_internal, user_with_internal};
use std::io;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => user_with_internal(
                "Could not find the requested file.",
                "Check that the file path you provided is correct and try again.",
                err,
            ),
            _ => system_with_internal(
                "An internal error occurred which we could not recover from.",
                "Please read the internal error below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                err,
            ),
        }
    }
}
