
use std::sync::Arc;
use super::{FileSource, Config, Launcher, Error, Resolver};

pub struct Core<F = super::DefaultFileSource, L = super::DefaultLauncher, R = super::DefaultResolver>
where F : FileSource, L : Launcher, R: Resolver
{
    pub config: Arc<Config>,
    pub file_source: Arc<F>,
    pub launcher: Arc<L>,
    pub resolver: Arc<R>
}

impl Core {
    pub fn builder() -> CoreBuilder<super::DefaultFileSource, super::DefaultLauncher, super::DefaultResolver> {
        CoreBuilder::default()
    }
}

pub struct CoreBuilder<F = super::DefaultFileSource, L = super::DefaultLauncher, R = super::DefaultResolver>
where F : FileSource, L : Launcher, R: Resolver
{
    config: Arc<Config>,
    file_source: Arc<F>,
    launcher: Arc<L>,
    resolver: Arc<R>
}

impl Default for CoreBuilder {
    fn default() -> Self {
        let config = Arc::new(Config::default());
        Self {
            config: config.clone(),
            file_source: Arc::new(super::DefaultFileSource::from(config.clone())),
            launcher: Arc::new(super::DefaultLauncher::from(config.clone())),
            resolver: Arc::new(super::DefaultResolver::from(config.clone()))
        }
    }
}

impl<F, L, R> std::convert::Into<Core<F, L, R>> for CoreBuilder<F, L, R>
where F : FileSource, L : Launcher, R: Resolver {
    fn into(self) -> Core<F, L, R> {
        self.build()
    }
}

impl<F, L, R> CoreBuilder<F, L, R>
where F : FileSource, L : Launcher, R: Resolver {
    pub fn build(&self) -> Core<F, L, R> {
        Core {
            config: self.config.clone(),
            file_source: self.file_source.clone(),
            launcher: self.launcher.clone(),
            resolver: self.resolver.clone()
        }
    }

    pub fn with_config(self, config: &Config) -> Self {
        let c = Arc::new(config.clone());

        Self {
            config: c.clone(),
            file_source: Arc::new(F::from(c.clone())),
            launcher: Arc::new(L::from(c.clone())),
            resolver: Arc::new(R::from(c.clone()))
        }
    }

    pub fn with_config_file(self, cfg_file: &str) -> Result<Self, Error> {
        let cfg = Config::from_file(&std::path::PathBuf::from(cfg_file))?;

        Ok(self.with_config(&cfg))
    }
    
    #[cfg(test)]
    pub fn with_mock_file_source<S>(self, setup: S) -> CoreBuilder<super::files::mocks::TestFileSource, L, R>
    where S : FnOnce(&mut super::files::mocks::TestFileSource) {
        let mut file_source = super::files::mocks::TestFileSource::from(self.config.clone());
        setup(&mut file_source);

        CoreBuilder {
            config: self.config,
            file_source: Arc::new(file_source),
            launcher: self.launcher,
            resolver: self.resolver
        }
    }

    #[cfg(test)]
    pub fn with_mock_launcher<S>(self, setup: S) -> CoreBuilder<F, super::launcher::mocks::MockLauncher, R>
    where S : FnOnce(&mut super::launcher::mocks::MockLauncher) {
        let mut launcher = super::launcher::mocks::MockLauncher::from(self.config.clone());
        setup(&mut launcher);

        CoreBuilder {
            config: self.config,
            file_source: self.file_source,
            launcher: Arc::new(launcher),
            resolver: self.resolver
        }
    }

    
    #[cfg(test)]
    pub fn with_mock_resolver<S>(self, setup: S) -> CoreBuilder<F, L, super::resolver::mocks::MockResolver>
    where S : FnOnce(&mut super::resolver::mocks::MockResolver) {
        let mut resolver = super::resolver::mocks::MockResolver::from(self.config.clone());
        setup(&mut resolver);

        CoreBuilder {
            config: self.config,
            file_source: self.file_source,
            launcher: self.launcher,
            resolver: Arc::new(resolver)
        }
    }
}