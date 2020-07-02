
use std::sync::Arc;
use super::{Config, Launcher, Error, Resolver, KeyChain};

pub struct Core<K = super::DefaultKeyChain, L = super::DefaultLauncher, R = super::DefaultResolver>
where K: KeyChain, L : Launcher, R: Resolver
{
    pub config: Arc<Config>,
    pub launcher: Arc<L>,
    pub resolver: Arc<R>,
    pub keychain: Arc<K>
}

impl Core {
    pub fn builder() -> CoreBuilder<super::DefaultKeyChain, super::DefaultLauncher, super::DefaultResolver> {
        CoreBuilder::default()
    }
}

pub struct CoreBuilder<K = super::DefaultKeyChain, L = super::DefaultLauncher, R = super::DefaultResolver>
where K: KeyChain, L : Launcher, R: Resolver
{
    config: Arc<Config>,
    launcher: Arc<L>,
    resolver: Arc<R>,
    keychain: Arc<K>
}

impl Default for CoreBuilder {
    fn default() -> Self {
        let config = Arc::new(Config::default());
        Self {
            config: config.clone(),
            launcher: Arc::new(super::DefaultLauncher::from(config.clone())),
            resolver: Arc::new(super::DefaultResolver::from(config.clone())),
            keychain: Arc::new(super::DefaultKeyChain::from(config.clone()))
        }
    }
}

impl<K, L, R> std::convert::Into<Core<K, L, R>> for CoreBuilder<K, L, R>
where K : KeyChain, L : Launcher, R: Resolver {
    fn into(self) -> Core<K, L, R> {
        self.build()
    }
}

impl<K, L, R> CoreBuilder<K, L, R>
where L : Launcher, R: Resolver, K: KeyChain {
    pub fn build(&self) -> Core<K, L, R> {
        Core {
            config: self.config.clone(),
            launcher: self.launcher.clone(),
            resolver: self.resolver.clone(),
            keychain: self.keychain.clone(),
        }
    }

    pub fn with_config(self, config: &Config) -> Self {
        let c = Arc::new(config.clone());

        Self {
            config: c.clone(),
            launcher: Arc::new(L::from(c.clone())),
            resolver: Arc::new(R::from(c.clone())),
            keychain: Arc::new(K::from(c.clone()))
        }
    }

    pub fn with_config_file(self, cfg_file: &str) -> Result<Self, Error> {
        let cfg = Config::from_file(&std::path::PathBuf::from(cfg_file))?;

        Ok(self.with_config(&cfg))
    }

    #[cfg(test)]
    pub fn with_mock_launcher<S>(self, setup: S) -> CoreBuilder<K, super::launcher::mocks::MockLauncher, R>
    where S : FnOnce(&mut super::launcher::mocks::MockLauncher) {
        let mut launcher = super::launcher::mocks::MockLauncher::from(self.config.clone());
        setup(&mut launcher);

        CoreBuilder {
            config: self.config,
            launcher: Arc::new(launcher),
            resolver: self.resolver,
            keychain: self.keychain
        }
    }

    
    #[cfg(test)]
    pub fn with_mock_resolver<S>(self, setup: S) -> CoreBuilder<K, L, super::resolver::mocks::MockResolver>
    where S : FnOnce(&mut super::resolver::mocks::MockResolver) {
        let mut resolver = super::resolver::mocks::MockResolver::from(self.config.clone());
        setup(&mut resolver);

        CoreBuilder {
            config: self.config,
            launcher: self.launcher,
            resolver: Arc::new(resolver),
            keychain: self.keychain
        }
    }
}