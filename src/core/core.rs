use super::{Config, Error, Input, KeyChain, Launcher, Output, Resolver};
use std::sync::Arc;

pub trait Core: Send + Sync {
    type Input: Input;
    type Output: Output;
    type HyperConnector: hyper::client::connect::Connect + Clone + Send + Sync + 'static;

    fn config(&self) -> &Config;
    fn keychain(&self) -> &KeyChain;
    fn launcher(&self) -> &Launcher;
    fn resolver(&self) -> &Resolver;
    fn input(&self) -> &Self::Input;
    fn output(&self) -> &Self::Output;
    fn http_client(&self) -> &hyper::Client<Self::HyperConnector>;
}

pub struct DefaultCore<
    I = super::DefaultInput,
    O = super::DefaultOutput,
    HC = hyper_tls::HttpsConnector<hyper::client::HttpConnector>,
> where
    I: Input,
    O: Output,
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    config: Arc<Config>,
    launcher: Arc<Launcher>,
    resolver: Arc<Resolver>,
    keychain: Arc<KeyChain>,
    input: Arc<I>,
    output: Arc<O>,
    http_client: Arc<hyper::Client<HC, hyper::Body>>,
}

impl<I, O, HC> Core for DefaultCore<I, O, HC>
where
    I: Input,
    O: Output,
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    type Input = I;
    type Output = O;
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

    fn input(&self) -> &Self::Input {
        &self.input
    }

    fn output(&self) -> &Self::Output {
        &self.output
    }

    fn http_client(&self) -> &hyper::Client<Self::HyperConnector> {
        &self.http_client
    }
}

pub struct CoreBuilder<
    I = super::DefaultInput,
    O = super::DefaultOutput,
    HC = hyper_tls::HttpsConnector<hyper::client::HttpConnector>,
> where
    I: Input,
    O: Output,
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    config: Arc<Config>,
    input: Arc<I>,
    output: Arc<O>,
    http_connector: HC,
}

impl Default for CoreBuilder {
    fn default() -> Self {
        let config = Arc::new(Config::default());
        Self {
            config: config.clone(),
            input: Arc::new(super::DefaultInput::from(config.clone())),
            output: Arc::new(super::DefaultOutput::from(config.clone())),
            http_connector: hyper_tls::HttpsConnector::<hyper::client::HttpConnector>::new(),
        }
    }
}

impl<I, O> std::convert::Into<DefaultCore<I, O>> for CoreBuilder<I, O>
where
    I: Input,
    O: Output,
{
    fn into(self) -> DefaultCore<I, O> {
        self.build()
    }
}

impl<I, O, HC> CoreBuilder<I, O, HC>
where
    I: Input,
    O: Output,
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    pub fn build(self) -> DefaultCore<I, O, HC> {
        DefaultCore {
            config: self.config.clone(),
            launcher: Arc::new(Launcher::from(self.config.clone())),
            resolver: Arc::new(Resolver::from(self.config.clone())),
            keychain: Arc::new(KeyChain::from(self.config.clone())),
            input: self.input,
            output: self.output,
            http_client: Arc::new(hyper::Client::builder().build(self.http_connector)),
        }
    }

    pub fn with_config(self, config: &Config) -> Self {
        let c = Arc::new(config.clone());

        Self {
            config: c.clone(),
            input: Arc::new(I::from(c.clone())),
            output: self.output,
            http_connector: self.http_connector,
        }
    }

    pub fn with_config_file(self, cfg_file: &str) -> Result<Self, Error> {
        let cfg = Config::from_file(&std::path::PathBuf::from(cfg_file))?;

        Ok(self.with_config(&cfg))
    }

    #[cfg(test)]
    pub fn with_mock_output(self) -> CoreBuilder<I, super::output::mocks::MockOutput, HC> {
        let output = super::output::mocks::MockOutput::from(self.config.clone());

        CoreBuilder {
            config: self.config,
            input: self.input,
            output: Arc::new(output),
            http_connector: self.http_connector,
        }
    }

    #[cfg(test)]
    pub fn with_mock_input<S>(self, setup: S) -> CoreBuilder<super::input::mocks::MockInput, O, HC>
    where
        S: FnOnce(&mut super::input::mocks::MockInput),
    {
        let mut input = super::input::mocks::MockInput::from(self.config.clone());
        setup(&mut input);

        CoreBuilder {
            config: self.config,
            input: Arc::new(input),
            output: self.output,
            http_connector: self.http_connector,
        }
    }

    #[cfg(test)]
    pub fn with_http_connector<S: hyper::client::connect::Connect>(
        self,
        connector: S,
    ) -> CoreBuilder<I, O, S>
    where
        S: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        CoreBuilder {
            config: self.config,
            input: self.input,
            output: self.output,
            http_connector: connector,
        }
    }
}
