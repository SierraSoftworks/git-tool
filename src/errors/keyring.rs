use super::*;
use ::keyring::KeyringError;

impl From<KeyringError> for Error {
    fn from(e: KeyringError) -> Self {
        match e {
            KeyringError::NoPasswordFound => user(
                "No access token could be found for the required service.",
                "Please make sure you have configured an access token using `git-tool auth github.com"),
            KeyringError::NoBackendFound => system(
                "No backend could be found to store secure tokens in.",
                "This likely means that your operating system is not supported by our keychain implementation. Please open an issue on GitHub and we'll see if we can help."),
            e => system_with_internal(
                "A problem occurred while trying to access the secure token store.",
                "This might indicate that you haven't configured an access token yet, which you can do with `git-tool auth github.com`. It may also indicate that there is an issue with your system secure token store. Please open a GitHub issue if you cannot resolve this.",
                detailed_message(&format!("{:?}", e)))
        }
    }
}