use std::io;

impl From<io::Error> for super::Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => human_errors::wrap_user(
                err,
                "Could not find the requested file.",
                &["Check that the file path you provided is correct and try again."],
            ),
            _ => human_errors::wrap_system(
                err,
                "An internal error occurred which we could not recover from.",
                &["Please read the internal error below and decide if there is something you can do to fix the problem, or report it to us on GitHub."],
            ),
        }
    }
}
