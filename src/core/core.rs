
use std::sync::Arc;
use super::{Config, Launcher, Error, Resolver, KeyChain, Output};

pub struct Core<K = super::DefaultKeyChain, L = super::DefaultLauncher, R = super::DefaultResolver, O = super::DefaultOutput>
where K: KeyChain, L : Launcher, R: Resolver, O: Output
{
    pub config: Arc<Config>,
    pub launcher: Arc<L>,
    pub resolver: Arc<R>,
    pub keychain: Arc<K>,
    pub output: Arc<O>
}

impl Core {
    pub fn builder() -> CoreBuilder<super::DefaultKeyChain, super::DefaultLauncher, super::DefaultResolver, super::DefaultOutput> {
        CoreBuilder::default()
    }
}

pub struct CoreBuilder<K = super::DefaultKeyChain, L = super::DefaultLauncher, R = super::DefaultResolver, O = super::DefaultOutput>
where K: KeyChain, L : Launcher, R: Resolver, O: Output
{
    config: Arc<Config>,
    launcher: Arc<L>,
    resolver: Arc<R>,
    keychain: Arc<K>,
    output: Arc<O>
}

impl Default for CoreBuilder {
    fn default() -> Self {
        let config = Arc::new(Config::default());
        Self {
            config: config.clone(),
            launcher: Arc::new(super::DefaultLauncher::from(config.clone())),
            resolver: Arc::new(super::DefaultResolver::from(config.clone())),
            keychain: Arc::new(super::DefaultKeyChain::from(config.clone())),
            output: Arc::new(super::DefaultOutput::from(config.clone()))
        }
    }
}

impl<K, L, R, O> std::convert::Into<Core<K, L, R, O>> for CoreBuilder<K, L, R, O>
where K : KeyChain, L : Launcher, R: Resolver, O: Output {
    fn into(self) -> Core<K, L, R, O> {
        self.build()
    }
}

impl<K, L, R, O> CoreBuilder<K, L, R, O>
where L : Launcher, R: Resolver, K: KeyChain, O: Output {
    pub fn build(&self) -> Core<K, L, R, O> {
        Core {
            config: self.config.clone(),
            launcher: self.launcher.clone(),
            resolver: self.resolver.clone(),
            keychain: self.keychain.clone(),
            output: self.output.clone()
        }
    }

    pub fn with_config(self, config: &Config) -> Self {
        let c = Arc::new(config.clone());

        Self {
            config: c.clone(),
            launcher: Arc::new(L::from(c.clone())),
            resolver: Arc::new(R::from(c.clone())),
            keychain: Arc::new(K::from(c.clone())),
            output: self.output
        }
    }

    pub fn with_config_file(self, cfg_file: &str) -> Result<Self, Error> {
        let cfg = Config::from_file(&std::path::PathBuf::from(cfg_file))?;

        Ok(self.with_config(&cfg))
    }

    #[cfg(test)]
    pub fn with_mock_keychain<S>(self, setup: S) -> CoreBuilder<super::auth::mocks::MockKeyChain, L, R, O>
    where S : FnOnce(&mut super::auth::mocks::MockKeyChain) {
        let mut keychain = super::auth::mocks::MockKeyChain::from(self.config.clone());
        setup(&mut keychain);

        CoreBuilder {
            config: self.config,
            launcher: self.launcher,
            resolver: self.resolver,
            keychain: Arc::new(keychain),
            output: self.output
        }
    }

    #[cfg(test)]
    pub fn with_mock_launcher<S>(self, setup: S) -> CoreBuilder<K, super::launcher::mocks::MockLauncher, R, O>
    where S : FnOnce(&mut super::launcher::mocks::MockLauncher) {
        let mut launcher = super::launcher::mocks::MockLauncher::from(self.config.clone());
        setup(&mut launcher);

        CoreBuilder {
            config: self.config,
            launcher: Arc::new(launcher),
            resolver: self.resolver,
            keychain: self.keychain,
            output: self.output
        }
    }

    #[cfg(test)]
    pub fn with_mock_resolver<S>(self, setup: S) -> CoreBuilder<K, L, super::resolver::mocks::MockResolver, O>
    where S : FnOnce(&mut super::resolver::mocks::MockResolver) {
        let mut resolver = super::resolver::mocks::MockResolver::from(self.config.clone());
        setup(&mut resolver);

        CoreBuilder {
            config: self.config,
            launcher: self.launcher,
            resolver: Arc::new(resolver),
            keychain: self.keychain,
            output: self.output
        }
    }

    #[cfg(test)]
    pub fn with_mock_output(self) -> CoreBuilder<K, L, R, super::output::mocks::MockOutput> {
        let output = super::output::mocks::MockOutput::from(self.config.clone());
        
        CoreBuilder {
            config: self.config,
            launcher: self.launcher,
            resolver: self.resolver,
            keychain: self.keychain,
            output: Arc::new(output)
        }
    }
}