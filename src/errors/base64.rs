use super::{system_with_internal, Error};

impl std::convert::From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        system_with_internal(
            "We could not decode a base64 response correctly.",
            "Please read the internal error below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
            err)
    }
}
