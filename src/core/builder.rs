use super::{Config, Core, Error, HttpClient, KeyChain, Launcher, Resolver};
use std::{path::PathBuf, sync::Arc};

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

    pub fn with_config_file<P: Into<PathBuf>>(self, cfg_file: P) -> Result<Self, Error> {
        let cfg = Config::from_file(&cfg_file.into())?;

        Ok(self.with_config(&cfg))
    }

    pub fn with_config_file_or_default<P: Into<PathBuf>>(self, cfg_file: P) -> Self {
        self.with_config(&Config::from_file_or_default(&cfg_file.into()))
    }
}
