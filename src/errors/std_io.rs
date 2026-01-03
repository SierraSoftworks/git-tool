use std::io;

impl super::HumanErrorExt for io::Error {
    fn to_human_error(self) -> human_errors::Error {
        match self.kind() {
            io::ErrorKind::NotFound => human_errors::wrap_user(
                self,
                "Could not find the requested file.",
                &["Check that the file path you provided is correct and try again."],
            ),
            _ => human_errors::wrap_system(
                self,
                "An internal error occurred which we could not recover from.",
                &[
                    "Please read the internal error below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                ],
            ),
        }
    }
}
