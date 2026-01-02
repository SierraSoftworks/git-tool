use ::keyring;

impl From<keyring::Error> for super::Error {
    fn from(e: keyring::Error) -> Self {
        match e {
            keyring::Error::NoEntry => human_errors::user(
                "No access token could be found for the required service.",
                &["Please make sure you have configured an access token using `git-tool auth gh`"],
            ),
            keyring::Error::NoStorageAccess(plat_err) => human_errors::wrap_system(
                plat_err,
                "No backend could be found to store secure tokens in.",
                &["This likely means that your operating system is not supported by our key-chain implementation. Please open an issue on GitHub and we'll see if we can help."],
            ),
            e => human_errors::wrap_user(
                super::StringError::new(format!("{e:?}")),
                "A problem occurred while trying to access the secure token store.",
                &["This might indicate that you haven't configured an access token yet, which you can do with `git-tool auth gh`. It may also indicate that there is an issue with your system secure token store. Please open a GitHub issue if you cannot resolve this."],
            ),
        }
    }
}
