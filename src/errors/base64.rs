impl super::HumanErrorExt for base64::DecodeError {
    fn to_human_error(self) -> human_errors::Error {
        human_errors::wrap_system(
            self,
            "We could not decode a base64 response correctly.",
            &[
                "Please read the internal error below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
            ],
        )
    }
}
