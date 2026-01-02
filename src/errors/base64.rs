impl From<base64::DecodeError> for super::Error {
    fn from(err: base64::DecodeError) -> Self {
        human_errors::wrap_system(
            err,
            "We could not decode a base64 response correctly.",
            &["Please read the internal error below and decide if there is something you can do to fix the problem, or report it to us on GitHub."],
        )
    }
}
