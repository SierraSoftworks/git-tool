use std::sync::Arc;
use keyring::Keyring;
use super::*;

pub trait KeyChain: From<Arc<Config>> + Send + Sync {
    fn get_token(&self, service: &str) -> Result<String, Error>;
    fn set_token(&self, service: &str, token: &str) -> Result<(), Error>;
    fn remove_token(&self, service: &str) -> Result<(), Error>;
}

pub struct SystemKeyChain {

}

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
    fn test_keychain() {
        let config = Arc::new(Config::default());
        let keychain = SystemKeyChain::from(config);

        assert!(keychain.get_token("test.example.com/missing").is_err());

        keychain.set_token("test.example.com/present", "example-token").unwrap();
        assert_eq!(keychain.get_token("test.example.com/present").unwrap(), "example-token");
        keychain.remove_token("test.example.com/present").unwrap();
        assert!(keychain.get_token("test.example.com/present").is_err());
    }
}