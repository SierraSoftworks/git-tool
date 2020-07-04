use super::{user_with_internal, Error};
use std::string::FromUtf8Error;

impl std::convert::From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        user_with_internal(
            "We could not parse the UTF-8 content we received.",
            "Make sure that you are not providing git-tool with content which is invalid UTF-8.",
            err,
        )
    }
}
