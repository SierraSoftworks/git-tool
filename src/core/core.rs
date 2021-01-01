use super::{Config, Error, HttpClient, KeyChain, Launcher, Resolver};
use std::{io::Write, sync::Arc};

#[cfg(test)]
use mocktopus::macros::*;

#[cfg_attr(test, mockable)]
pub struct Core {
    config: Arc<Config>,
    launcher: Arc<Launcher>,
    resolver: Arc<Resolver>,
    keychain: Arc<KeyChain>,
    http_client: Arc<HttpClient>,
}

#[cfg_attr(test, mockable)]
impl Core {
    pub fn builder() -> CoreBuilder {
        let config = Arc::new(Config::default());
        CoreBuilder { config }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn keychain(&self) -> &KeyChain {
        &self.keychain
    }

    pub fn launcher(&self) -> &Launcher {
        &self.launcher
    }

    pub fn resolver(&self) -> &Resolver {
        &self.resolver
    }

    pub fn output(&self) -> Box<dyn Write + Send> {
        crate::console::output::output()
    }

    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }
}

pub struct CoreBuilder {
    config: Arc<Config>,
}

impl std::convert::Into<Core> for CoreBuilder {
    fn into(self) -> Core {
        self.build()
    }
}

impl CoreBuilder {
    pub fn build(self) -> Core {
        Core {
            config: self.config.clone(),
            launcher: Arc::new(Launcher::from(self.config.clone())),
            resolver: Arc::new(Resolver::from(self.config.clone())),
            keychain: Arc::new(KeyChain::from(self.config.clone())),
            http_client: Arc::new(HttpClient::from(self.config.clone())),
        }
    }

    pub fn with_config(self, config: &Config) -> Self {
        let c = Arc::new(config.clone());

        Self { config: c.clone() }
    }

    pub fn with_config_file(self, cfg_file: &str) -> Result<Self, Error> {
        let cfg = Config::from_file(&std::path::PathBuf::from(cfg_file))?;

        Ok(self.with_config(&cfg))
    }
}
