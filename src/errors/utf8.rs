use std::string::FromUtf8Error;

impl super::HumanErrorExt for FromUtf8Error {
    fn to_human_error(self) -> human_errors::Error {
        human_errors::wrap_user(
            self,
            "We could not parse the UTF-8 content we received.",
            &["Make sure that you are not providing git-tool with content which is invalid UTF-8."],
        )
    }
}
