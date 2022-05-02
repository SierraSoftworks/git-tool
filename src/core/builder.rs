use super::{Config, Core, Error, HttpClient, KeyChain, Launcher, Resolver};
use std::sync::Arc;

pub struct CoreBuilder {
    pub(super) config: Arc<Config>,
}

impl std::convert::From<CoreBuilder> for Core {
    fn from(builder: CoreBuilder) -> Core {
        builder.build()
    }
}

impl CoreBuilder {
    pub fn build(self) -> Core {
        Core {
            config: self.config.clone(),
            launcher: Arc::new(Launcher::from(self.config.clone())),
            resolver: Arc::new(Resolver::from(self.config.clone())),
            keychain: Arc::new(KeyChain::from(self.config.clone())),
            http_client: Arc::new(HttpClient::from(self.config)),
        }
    }

    pub fn with_config(self, config: &Config) -> Self {
        let c = Arc::new(config.clone());

        Self { config: c }
    }

    pub fn with_config_file(self, cfg_file: &str) -> Result<Self, Error> {
        let cfg = Config::from_file(&std::path::PathBuf::from(cfg_file))?;

        Ok(self.with_config(&cfg))
    }
}
