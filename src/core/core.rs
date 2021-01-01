use super::{Config, Error, KeyChain, Launcher, Resolver};
use std::{
    io::{Read, Write},
    sync::Arc,
};

pub trait Core: Send + Sync {
    type HyperConnector: hyper::client::connect::Connect + Clone + Send + Sync + 'static;

    fn config(&self) -> &Config;
    fn keychain(&self) -> &KeyChain;
    fn launcher(&self) -> &Launcher;
    fn resolver(&self) -> &Resolver;
    fn input(&self) -> Box<dyn Read + Send>;
    fn output(&self) -> Box<dyn Write + Send>;
    fn http_client(&self) -> &hyper::Client<Self::HyperConnector>;
}

pub struct DefaultCore<HC = hyper_tls::HttpsConnector<hyper::client::HttpConnector>>
where
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    config: Arc<Config>,
    launcher: Arc<Launcher>,
    resolver: Arc<Resolver>,
    keychain: Arc<KeyChain>,
    http_client: Arc<hyper::Client<HC, hyper::Body>>,
}

impl<HC> Core for DefaultCore<HC>
where
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    type HyperConnector = HC;

    fn config(&self) -> &Config {
        &self.config
    }

    fn keychain(&self) -> &KeyChain {
        &self.keychain
    }

    fn launcher(&self) -> &Launcher {
        &self.launcher
    }

    fn resolver(&self) -> &Resolver {
        &self.resolver
    }

    fn input(&self) -> Box<dyn Read + Send> {
        crate::console::input::input()
    }

    fn output(&self) -> Box<dyn Write + Send> {
        crate::console::output::output()
    }

    fn http_client(&self) -> &hyper::Client<Self::HyperConnector> {
        &self.http_client
    }
}

pub struct CoreBuilder<HC = hyper_tls::HttpsConnector<hyper::client::HttpConnector>>
where
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    config: Arc<Config>,
    http_connector: HC,
}

impl Default for CoreBuilder {
    fn default() -> Self {
        let config = Arc::new(Config::default());
        Self {
            config: config.clone(),
            http_connector: hyper_tls::HttpsConnector::<hyper::client::HttpConnector>::new(),
        }
    }
}

impl std::convert::Into<DefaultCore> for CoreBuilder {
    fn into(self) -> DefaultCore {
        self.build()
    }
}

impl<HC> CoreBuilder<HC>
where
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    pub fn build(self) -> DefaultCore<HC> {
        DefaultCore {
            config: self.config.clone(),
            launcher: Arc::new(Launcher::from(self.config.clone())),
            resolver: Arc::new(Resolver::from(self.config.clone())),
            keychain: Arc::new(KeyChain::from(self.config.clone())),
            http_client: Arc::new(hyper::Client::builder().build(self.http_connector)),
        }
    }

    pub fn with_config(self, config: &Config) -> Self {
        let c = Arc::new(config.clone());

        Self {
            config: c.clone(),
            http_connector: self.http_connector,
        }
    }

    pub fn with_config_file(self, cfg_file: &str) -> Result<Self, Error> {
        let cfg = Config::from_file(&std::path::PathBuf::from(cfg_file))?;

        Ok(self.with_config(&cfg))
    }

    #[cfg(test)]
    pub fn with_http_connector<S: hyper::client::connect::Connect>(
        self,
        connector: S,
    ) -> CoreBuilder<S>
    where
        S: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        CoreBuilder {
            config: self.config,
            http_connector: connector,
        }
    }
}
