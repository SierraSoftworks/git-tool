
use std::sync::Arc;
use super::{FileSource, Config, Launcher, Error, Resolver};

pub struct Core<F = super::files::FileSystemSource, L = super::launcher::TokioLauncher, R = super::resolver::FileSystemResolver>
where F : FileSource, L : Launcher, R: Resolver
{
    pub config: Arc<Config>,
    pub file_source: Arc<F>,
    pub launcher: Arc<L>,
    pub resolver: Arc<R>
}

impl Core {
    pub fn builder() -> CoreBuilder<super::files::FileSystemSource, super::launcher::TokioLauncher, super::resolver::FileSystemResolver> {
        CoreBuilder::default()
    }
}

pub struct CoreBuilder<F = super::files::FileSystemSource, L = super::launcher::TokioLauncher, R = super::resolver::FileSystemResolver>
where F : FileSource, L : Launcher, R: Resolver
{
    config: Config,
    file_source: Option<Arc<F>>,
    launcher: Option<Arc<L>>,
    resolver: Option<Arc<R>>
}

impl Default for CoreBuilder {
    fn default() -> Self {
        Self {
            config: Config::default(),
            file_source: None,
            launcher: None,
            resolver: None
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
            config: Arc::new(self.config.clone()),
            file_source: self.file_source.clone().unwrap_or(Arc::new(F::from(self.config.clone()))),
            launcher: self.launcher.clone().unwrap_or(Arc::new(L::from(self.config.clone()))),
            resolver: self.resolver.clone().unwrap_or(Arc::new(R::from(self.config.clone())))
        }
    }

    pub fn with_config(self, config: &Config) -> Self {
        Self {
            config: config.clone(),
            file_source: self.file_source,
            launcher: self.launcher,
            resolver: self.resolver
        }
    }

    pub fn with_config_file(self, cfg_file: &str) -> Result<Self, Error> {
        let f = std::fs::File::open(cfg_file)?;

        let cfg = Config::from_reader(f)?;

        Ok(self.with_config(&cfg))
    }
    
    #[cfg(test)]
    pub fn with_mock_file_source<S>(self, setup: S) -> CoreBuilder<super::files::TestFileSource, L, R>
    where S : FnOnce(&mut super::files::TestFileSource) {
        let mut file_source = super::files::TestFileSource::from(self.config.clone());
        setup(&mut file_source);

        CoreBuilder {
            config: self.config,
            file_source: Some(Arc::new(file_source)),
            launcher: self.launcher,
            resolver: self.resolver
        }
    }

    #[cfg(test)]
    pub fn with_mock_launcher<S>(self, setup: S) -> CoreBuilder<F, super::launcher::MockLauncher, R>
    where S : FnOnce(&mut super::launcher::MockLauncher) {
        let mut launcher = super::MockLauncher::from(self.config.clone());
        setup(&mut launcher);

        CoreBuilder {
            config: self.config,
            file_source: self.file_source,
            launcher: Some(Arc::new(launcher)),
            resolver: self.resolver
        }
    }

    
    #[cfg(test)]
    pub fn with_mock_resolver<S>(self, setup: S) -> CoreBuilder<F, L, super::resolver::MockResolver>
    where S : FnOnce(&mut super::resolver::MockResolver) {
        let mut resolver = super::resolver::MockResolver::from(self.config.clone());
        setup(&mut resolver);

        CoreBuilder {
            config: self.config,
            file_source: self.file_source,
            launcher: self.launcher,
            resolver: Some(Arc::new(resolver))
        }
    }
}