use super::*;
use std::sync::Arc;

#[cfg(feature = "auth")]
use keyring::Keyring;

#[cfg(test)]
use mocktopus::macros::*;

#[cfg_attr(test, mockable)]
pub struct KeyChain {}

impl From<Arc<Config>> for KeyChain {
    fn from(_: Arc<Config>) -> Self {
        Self {}
    }
}

#[cfg(feature = "auth")]
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

#[cfg(not(feature = "auth"))]
#[cfg_attr(test, mockable)]
#[allow(dead_code)]
impl KeyChain {
    pub fn get_token(&self, _service: &str) -> Result<String, Error> {
        Err(errors::user(
            "This version of Git-Tool was compiled without support for authentication.",
            "Use a version of Git-Tool which supports authentication, or compile Git-Tool yourself with --features=auth.",
        ))
    }

    pub fn set_token(&self, _service: &str, _token: &str) -> Result<(), Error> {
        Ok(())
    }

    pub fn remove_token(&self, _service: &str) -> Result<(), Error> {
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
