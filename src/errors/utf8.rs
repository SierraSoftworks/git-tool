use std::string::FromUtf8Error;

impl From<FromUtf8Error> for super::Error {
    fn from(err: FromUtf8Error) -> Self {
        human_errors::wrap_user(
            err,
            "We could not parse the UTF-8 content we received.",
            &["Make sure that you are not providing git-tool with content which is invalid UTF-8."],
        )
    }
}
