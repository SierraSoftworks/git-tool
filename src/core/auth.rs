use super::*;
use keyring::Keyring;
use std::sync::Arc;

pub trait KeyChain: From<Arc<Config>> + Send + Sync {
    fn get_token(&self, service: &str) -> Result<String, Error>;
    fn set_token(&self, service: &str, token: &str) -> Result<(), Error>;
    fn remove_token(&self, service: &str) -> Result<(), Error>;
}

pub struct SystemKeyChain {}

impl From<Arc<Config>> for SystemKeyChain {
    fn from(_: Arc<Config>) -> Self {
        Self {}
    }
}

impl KeyChain for SystemKeyChain {
    fn get_token(&self, service: &str) -> Result<String, Error> {
        let token = Keyring::new("git-tool", service).get_password()?;

        Ok(token)
    }

    fn set_token(&self, service: &str, token: &str) -> Result<(), Error> {
        Keyring::new("git-tool", service).set_password(token)?;
        Ok(())
    }

    fn remove_token(&self, service: &str) -> Result<(), Error> {
        Keyring::new("git-tool", service).delete_password()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_keychain() {
        let config = Arc::new(Config::default());
        let keychain = SystemKeyChain::from(config);

        assert!(keychain.get_token("test.example.com/missing").is_err());

        keychain
            .set_token("test.example.com/present", "example-token")
            .unwrap();
        assert_eq!(
            keychain.get_token("test.example.com/present").unwrap(),
            "example-token"
        );
        keychain.remove_token("test.example.com/present").unwrap();
        assert!(keychain.get_token("test.example.com/present").is_err());
    }
}

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::{collections::BTreeMap, sync::Mutex};

    pub struct MockKeyChain {
        tokens: Arc<Mutex<BTreeMap<String, String>>>,
    }

    impl From<Arc<Config>> for MockKeyChain {
        fn from(_: Arc<Config>) -> Self {
            Self {
                tokens: Arc::new(Mutex::new(BTreeMap::new())),
            }
        }
    }

    impl KeyChain for MockKeyChain {
        fn get_token(&self, service: &str) -> Result<String, Error> {
            self.tokens
                .lock()
                .map_err(|_| {
                    errors::system(
                        "Could not read the token from the keychain due to a poisoned lock.",
                        "Please restart the application and try again.",
                    )
                })
                .and_then(|t| {
                    t.get(service).map(|o| o.clone()).ok_or(errors::user(
                        "Could not find an access token for this service.",
                        &format!(
                            "Please add an access token using `git-tool auth {}`.",
                            service
                        ),
                    ))
                })
        }

        fn set_token(&self, service: &str, token: &str) -> Result<(), Error> {
            self.tokens
                .lock()
                .map_err(|_| {
                    errors::system(
                        "Could not read the token from the keychain due to a poisoned lock.",
                        "Please restart the application and try again.",
                    )
                })
                .map(|mut t| {
                    t.insert(service.to_string(), token.to_string())
                        .unwrap_or_default()
                })
                .map(|_| ())
        }

        fn remove_token(&self, service: &str) -> Result<(), Error> {
            self.tokens
                .lock()
                .map_err(|_| {
                    errors::system(
                        "Could not read the token from the keychain due to a poisoned lock.",
                        "Please restart the application and try again.",
                    )
                })
                .map(|mut t| t.remove(service).unwrap_or_default())
                .map(|_| ())
        }
    }
}
