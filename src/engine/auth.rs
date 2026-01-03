use super::*;
use crate::errors::HumanErrorResultExt as _;
use tracing_batteries::prelude::*;

#[cfg(feature = "auth")]
#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait KeyChain: Send + Sync {
    fn get_token(&self, service: &str) -> Result<String, human_errors::Error>;
    fn set_token(&self, service: &str, token: &str) -> Result<(), human_errors::Error>;
    fn delete_token(&self, service: &str) -> Result<(), human_errors::Error>;
}

#[cfg(feature = "auth")]
pub fn keychain() -> Arc<dyn KeyChain + Send + Sync> {
    Arc::new(TrueKeyChain {})
}

#[cfg(not(feature = "auth"))]
pub fn keychain() -> Arc<dyn KeyChain + Send + Sync> {
    Arc::new(UnsupportedKeyChain {})
}
#[cfg(feature = "auth")]
struct TrueKeyChain {}

#[cfg(feature = "auth")]
impl KeyChain for TrueKeyChain {
    #[tracing::instrument(err, skip(self))]
    fn get_token(&self, service: &str) -> Result<String, human_errors::Error> {
        use crate::errors::HumanErrorResultExt;

        let token = keyring::Entry::new("git-tool", service)
            .to_human_error()?
            .get_password()
            .to_human_error()?;

        Ok(token)
    }

    #[tracing::instrument(err, skip(self, token))]
    fn set_token(&self, service: &str, token: &str) -> Result<(), human_errors::Error> {
        keyring::Entry::new("git-tool", service)
            .to_human_error()?
            .set_password(token)
            .to_human_error()?;
        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    fn delete_token(&self, service: &str) -> Result<(), human_errors::Error> {
        keyring::Entry::new("git-tool", service)
            .to_human_error()?
            .delete_credential()
            .to_human_error()?;
        Ok(())
    }
}

#[cfg(not(feature = "auth"))]
#[allow(dead_code)]
struct UnsupportedKeyChain {}

#[cfg(not(feature = "auth"))]
#[allow(dead_code)]
impl KeyChain for UnsupportedKeyChain {
    fn get_token(&self, _service: &str) -> Result<String, human_errors::Error> {
        Err(human_errors::user(
            "This version of Git-Tool was compiled without support for authentication.",
            &[
                "Use a version of Git-Tool which supports authentication, or compile Git-Tool yourself with --features=auth.",
            ],
        ))
    }

    fn set_token(&self, _service: &str, _token: &str) -> Result<(), human_errors::Error> {
        Ok(())
    }

    fn delete_token(&self, _service: &str) -> Result<(), human_errors::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_keychain() {
        let keychain = keychain();

        assert!(keychain.get_token("test.example.com/missing").is_err());

        keychain
            .set_token("test.example.com/present", "example-token")
            .unwrap();
        assert_eq!(
            keychain.get_token("test.example.com/present").unwrap(),
            "example-token"
        );
        keychain.delete_token("test.example.com/present").unwrap();
        assert!(keychain.get_token("test.example.com/present").is_err());
    }
}
