use super::*;

#[cfg(feature = "auth")]
#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait KeyChain: Send + Sync {
    fn get_token(&self, service: &str) -> Result<String, Error>;
    fn set_token(&self, service: &str, token: &str) -> Result<(), Error>;
    fn delete_token(&self, service: &str) -> Result<(), Error>;
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
    fn get_token(&self, service: &str) -> Result<String, Error> {
        let token = keyring::Entry::new("git-tool", service)?.get_password()?;

        Ok(token)
    }

    #[tracing::instrument(err, skip(self, token))]
    fn set_token(&self, service: &str, token: &str) -> Result<(), Error> {
        keyring::Entry::new("git-tool", service)?.set_password(token)?;
        Ok(())
    }

    #[tracing::instrument(err, skip(self))]
    fn delete_token(&self, service: &str) -> Result<(), Error> {
        keyring::Entry::new("git-tool", service)?.delete_credential()?;
        Ok(())
    }
}

#[cfg(not(feature = "auth"))]
#[allow(dead_code)]
struct UnsupportedKeyChain {}

#[cfg(not(feature = "auth"))]
#[allow(dead_code)]
impl KeyChain for UnsupportedKeyChain {
    fn get_token(&self, _service: &str) -> Result<String, Error> {
        Err(errors::user(
            "This version of Git-Tool was compiled without support for authentication.",
            "Use a version of Git-Tool which supports authentication, or compile Git-Tool yourself with --features=auth.",
        ))
    }

    fn set_token(&self, _service: &str, _token: &str) -> Result<(), Error> {
        Ok(())
    }

    fn delete_token(&self, _service: &str) -> Result<(), Error> {
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
