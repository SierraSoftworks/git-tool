
use std::sync::Arc;
use super::{FileSource, Config, Launcher, Error, Resolver};

pub struct Core {
    pub config: Arc<Config>,
    pub file_source: Arc<dyn FileSource + Sync + Send>,
    pub launcher: Arc<dyn Launcher + Sync + Send>,
    pub resolver: Arc<dyn Resolver + Sync + Send>
}

impl Core {
    pub fn builder() -> CoreBuilder {
        CoreBuilder::default()
    }
}

pub struct CoreBuilder {
    config: Arc<Config>,
    file_source: Arc<dyn FileSource + Sync + Send>,
    launcher: Arc<dyn Launcher + Sync + Send>,
    resolver: Arc<dyn Resolver + Sync + Send>
}

impl Default for CoreBuilder {
    fn default() -> Self {
        let cfg = Arc::new(Config::default());

        Self {
            config: cfg.clone(),
            file_source: Arc::new(super::files::FileSystemSource{}),
            launcher: Arc::new(super::launcher::TokioLauncher{}),
            resolver: Arc::new(super::resolver::FileSystemResolver::new(cfg.clone()))
        }
    }
}

impl std::convert::Into<Core> for CoreBuilder {
    fn into(self) -> Core {
        self.build()
    }
}

impl CoreBuilder {
    pub fn build(&self) -> Core {
        Core {
            config: self.config.clone(),
            file_source: self.file_source.clone(),
            launcher: self.launcher.clone(),
            resolver: self.resolver.clone()
        }
    }

    pub fn with_config(&mut self, config: &Config) -> &Self {
        self.config = Arc::new(config.clone());
        self
    }

    pub fn with_config_file(&mut self, cfg_file: &str) -> Result<&Self, Error> {
        let f = std::fs::File::open(cfg_file)?;

        let cfg = Config::from_reader(f)?;

        Ok(self.with_config(&cfg))
    }
    
    #[cfg(test)]
    pub fn with_file_source(&mut self, file_source: Arc<dyn FileSource + Send + Sync>) -> &Self {
        self.file_source = file_source;
        self
    }
    
    #[cfg(test)]
    pub fn with_launcher(&mut self, launcher: Arc<dyn Launcher + Send + Sync>) -> &Self {
        self.launcher = launcher;
        self
    }
    
    #[cfg(test)]
    pub fn with_resolver(&mut self, resolver: Arc<dyn Resolver + Send + Sync>) -> &Self {
        self.resolver = resolver;
        self
    }
}