impl From<nix::Error> for super::Error {
    fn from(err: nix::Error) -> Self {
        human_errors::wrap_system(
            err,
            "We could not send a signal to the child process correctly. This may impact the way Git-Tool responds to interrupts and termination signals.",
            &[
                "Please let us know what happened on GitHub so that we can help resolve the issue for you. Ensure that you provide us with information on your operating system and the version of Git-Tool you're running.",
            ],
        )
    }
}
