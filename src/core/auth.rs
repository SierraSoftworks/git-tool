use super::*;
use keyring::Keyring;
use std::sync::Arc;

#[cfg(test)]
use mocktopus::macros::*;

#[cfg_attr(test, mockable)]
pub struct KeyChain {}

impl From<Arc<Config>> for KeyChain {
    fn from(_: Arc<Config>) -> Self {
        Self {}
    }
}

#[cfg_attr(test, mockable)]
impl KeyChain {
    pub fn get_token(&self, service: &str) -> Result<String, Error> {
        let token = Keyring::new("git-tool", service).get_password()?;

        Ok(token)
    }

    pub fn set_token(&self, service: &str, token: &str) -> Result<(), Error> {
        Keyring::new("git-tool", service).set_password(token)?;
        Ok(())
    }

    pub fn remove_token(&self, service: &str) -> Result<(), Error> {
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
        let keychain = KeyChain::from(config);

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
