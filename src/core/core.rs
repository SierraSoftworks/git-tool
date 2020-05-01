
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
    config: Config,
    file_source: Option<Arc<dyn FileSource + Sync + Send>>,
    launcher: Option<Arc<dyn Launcher + Sync + Send>>,
    resolver: Option<Arc<dyn Resolver + Sync + Send>>
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

impl std::convert::Into<Core> for CoreBuilder {
    fn into(self) -> Core {
        self.build()
    }
}

impl CoreBuilder {
    pub fn build(&self) -> Core {
        Core {
            config: Arc::new(self.config.clone()),
            file_source: self.file_source.clone().unwrap_or(Arc::new(super::files::FileSystemSource{})),
            launcher: self.launcher.clone().unwrap_or(Arc::new(super::launcher::TokioLauncher{})),
            resolver: self.resolver.clone().unwrap_or(Arc::new(super::resolver::FileSystemResolver::new(self.config.clone())))
        }
    }

    pub fn with_config(&mut self, config: &Config) -> &mut Self {
        self.config = config.clone();
        self
    }

    pub fn with_config_file(&mut self, cfg_file: &str) -> Result<&mut Self, Error> {
        let f = std::fs::File::open(cfg_file)?;

        let cfg = Config::from_reader(f)?;

        Ok(self.with_config(&cfg))
    }
    
    #[cfg(test)]
    pub fn with_file_source(&mut self, file_source: Arc<dyn FileSource + Send + Sync>) -> &mut Self {
        self.file_source = Some(file_source);
        self
    }
    
    #[cfg(test)]
    pub fn with_launcher(&mut self, launcher: Arc<dyn Launcher + Send + Sync>) -> &mut Self {
        self.launcher = Some(launcher);
        self
    }
    
    #[cfg(test)]
    pub fn with_resolver(&mut self, resolver: Arc<dyn Resolver + Send + Sync>) -> &mut Self {
        self.resolver = Some(resolver);
        self
    }
}