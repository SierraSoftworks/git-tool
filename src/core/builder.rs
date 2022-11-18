use crate::console::{self, ConsoleProvider};

use super::{
    auth, http, launcher, resolver, Config, Core, Error, HttpClient, KeyChain, Launcher, Resolver,
};
use std::{path::PathBuf, sync::Arc};

#[cfg(test)]
use super::MockHttpRoute;

#[derive(Default)]
pub struct CoreBuilderWithoutConfig;

impl CoreBuilderWithoutConfig {
    pub fn with_default_config(self) -> CoreBuilderWithConfig {
        let config = Config::default();
        self.with_config(config)
    }

    pub fn with_config(self, config: Config) -> CoreBuilderWithConfig {
        let config = Arc::new(config);

        CoreBuilderWithConfig {
            launcher: launcher::launcher(config.clone()),
            resolver: resolver::resolver(config.clone()),
            keychain: auth::keychain(),
            http_client: http::client(),
            console: console::default(),
            config,
        }
    }

    pub fn with_config_file<P: Into<PathBuf>>(
        self,
        cfg_file: P,
    ) -> Result<CoreBuilderWithConfig, Error> {
        let cfg = Config::from_file(&cfg_file.into())?;

        Ok(self.with_config(cfg))
    }

    pub fn with_config_file_or_default<P: Into<PathBuf>>(
        self,
        cfg_file: P,
    ) -> CoreBuilderWithConfig {
        self.with_config(Config::from_file_or_default(&cfg_file.into()))
    }

    #[cfg(test)]
    pub fn with_config_for_dev_directory<P: Into<PathBuf>>(self, path: P) -> CoreBuilderWithConfig {
        let config = Config::for_dev_directory(&path.into());

        self.with_config(config)
    }
}

pub struct CoreBuilderWithConfig {
    pub(super) config: Arc<Config>,
    pub(super) console: Arc<dyn console::ConsoleProvider + Send + Sync>,
    pub(super) launcher: Arc<dyn Launcher + Send + Sync>,
    pub(super) resolver: Arc<dyn Resolver + Send + Sync>,
    pub(super) keychain: Arc<dyn KeyChain + Send + Sync>,
    pub(super) http_client: Arc<dyn HttpClient + Send + Sync>,
}

impl std::convert::From<CoreBuilderWithConfig> for Core {
    fn from(builder: CoreBuilderWithConfig) -> Core {
        builder.build()
    }
}

#[allow(dead_code)]
impl CoreBuilderWithConfig {
    pub fn build(self) -> Core {
        Core {
            config: self.config,
            launcher: self.launcher,
            resolver: self.resolver,
            keychain: self.keychain,
            http_client: self.http_client,
            console: self.console,
        }
    }

    pub fn with_console(self, console: Arc<dyn ConsoleProvider + Send + Sync>) -> Self {
        Self { console, ..self }
    }

    #[cfg(test)]
    pub fn with_null_console(self) -> Self {
        self.with_console(crate::console::null())
    }

    pub fn with_launcher(self, launcher: Arc<dyn Launcher + Send + Sync>) -> Self {
        Self { launcher, ..self }
    }

    #[cfg(test)]
    pub fn with_mock_launcher<F: FnMut(&mut launcher::MockLauncher)>(self, mut config: F) -> Self {
        let mut mock = launcher::MockLauncher::new();
        config(&mut mock);
        self.with_launcher(Arc::new(mock))
    }

    pub fn with_resolver(self, resolver: Arc<dyn Resolver + Send + Sync>) -> Self {
        Self { resolver, ..self }
    }

    #[cfg(test)]
    pub fn with_mock_resolver<F: FnMut(&mut resolver::MockResolver)>(self, mut config: F) -> Self {
        let mut mock = resolver::MockResolver::new();
        config(&mut mock);
        self.with_resolver(Arc::new(mock))
    }

    pub fn with_keychain(self, keychain: Arc<dyn KeyChain + Send + Sync>) -> Self {
        Self { keychain, ..self }
    }

    #[cfg(test)]
    pub fn with_mock_keychain<F: FnMut(&mut auth::MockKeyChain)>(self, mut config: F) -> Self {
        let mut mock = auth::MockKeyChain::new();
        config(&mut mock);
        self.with_keychain(Arc::new(mock))
    }

    pub fn with_http_client(self, http_client: Arc<dyn HttpClient + Send + Sync>) -> Self {
        Self {
            http_client,
            ..self
        }
    }

    #[cfg(test)]
    pub fn with_mock_http_client(self, routes: Vec<MockHttpRoute>) -> Self {
        self.with_http_client(http::mock(routes))
    }
}
