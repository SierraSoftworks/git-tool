use super::{Config, Error, Input, KeyChain, Launcher, Output, Resolver};
use std::sync::Arc;

pub trait Core: Send + Sync {
    type KeyChain: KeyChain;
    type Resolver: Resolver;
    type Input: Input;
    type Output: Output;
    type HyperConnector: hyper::client::connect::Connect + Clone + Send + Sync + 'static;

    fn config(&self) -> &Config;
    fn keychain(&self) -> &Self::KeyChain;
    fn launcher(&self) -> &Launcher;
    fn resolver(&self) -> &Self::Resolver;
    fn input(&self) -> &Self::Input;
    fn output(&self) -> &Self::Output;
    fn http_client(&self) -> &hyper::Client<Self::HyperConnector>;
}

pub struct DefaultCore<
    K = super::DefaultKeyChain,
    R = super::DefaultResolver,
    I = super::DefaultInput,
    O = super::DefaultOutput,
    HC = hyper_tls::HttpsConnector<hyper::client::HttpConnector>,
> where
    K: KeyChain,
    R: Resolver,
    I: Input,
    O: Output,
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    config: Arc<Config>,
    launcher: Arc<Launcher>,
    resolver: Arc<R>,
    keychain: Arc<K>,
    input: Arc<I>,
    output: Arc<O>,
    http_client: Arc<hyper::Client<HC, hyper::Body>>,
}

impl<K, R, I, O, HC> Core for DefaultCore<K, R, I, O, HC>
where
    K: KeyChain,
    R: Resolver,
    I: Input,
    O: Output,
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    type KeyChain = K;
    type Resolver = R;
    type Input = I;
    type Output = O;
    type HyperConnector = HC;

    fn config(&self) -> &Config {
        &self.config
    }

    fn keychain(&self) -> &Self::KeyChain {
        &self.keychain
    }

    fn launcher(&self) -> &Launcher {
        &self.launcher
    }

    fn resolver(&self) -> &Self::Resolver {
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
    K = super::DefaultKeyChain,
    R = super::DefaultResolver,
    I = super::DefaultInput,
    O = super::DefaultOutput,
    HC = hyper_tls::HttpsConnector<hyper::client::HttpConnector>,
> where
    K: KeyChain,
    R: Resolver,
    I: Input,
    O: Output,
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    config: Arc<Config>,
    resolver: Arc<R>,
    keychain: Arc<K>,
    input: Arc<I>,
    output: Arc<O>,
    http_connector: HC,
}

impl Default for CoreBuilder {
    fn default() -> Self {
        let config = Arc::new(Config::default());
        Self {
            config: config.clone(),
            resolver: Arc::new(super::DefaultResolver::from(config.clone())),
            keychain: Arc::new(super::DefaultKeyChain::from(config.clone())),
            input: Arc::new(super::DefaultInput::from(config.clone())),
            output: Arc::new(super::DefaultOutput::from(config.clone())),
            http_connector: hyper_tls::HttpsConnector::<hyper::client::HttpConnector>::new(),
        }
    }
}

impl<K, R, I, O> std::convert::Into<DefaultCore<K, R, I, O>> for CoreBuilder<K, R, I, O>
where
    K: KeyChain,
    R: Resolver,
    I: Input,
    O: Output,
{
    fn into(self) -> DefaultCore<K, R, I, O> {
        self.build()
    }
}

impl<K, R, I, O, HC> CoreBuilder<K, R, I, O, HC>
where
    R: Resolver,
    K: KeyChain,
    I: Input,
    O: Output,
    HC: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
    pub fn build(self) -> DefaultCore<K, R, I, O, HC> {
        DefaultCore {
            config: self.config.clone(),
            launcher: Arc::new(Launcher::from(self.config.clone())),
            resolver: self.resolver,
            keychain: self.keychain,
            input: self.input,
            output: self.output,
            http_client: Arc::new(hyper::Client::builder().build(self.http_connector)),
        }
    }

    pub fn with_config(self, config: &Config) -> Self {
        let c = Arc::new(config.clone());

        Self {
            config: c.clone(),
            resolver: Arc::new(R::from(c.clone())),
            keychain: Arc::new(K::from(c.clone())),
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
    pub fn with_mock_keychain<S>(
        self,
        setup: S,
    ) -> CoreBuilder<super::auth::mocks::MockKeyChain, R, I, O, HC>
    where
        S: FnOnce(&mut super::auth::mocks::MockKeyChain),
    {
        let mut keychain = super::auth::mocks::MockKeyChain::from(self.config.clone());
        setup(&mut keychain);

        CoreBuilder {
            config: self.config,
            resolver: self.resolver,
            keychain: Arc::new(keychain),
            input: self.input,
            output: self.output,
            http_connector: self.http_connector,
        }
    }

    #[cfg(test)]
    pub fn with_mock_resolver<S>(
        self,
        setup: S,
    ) -> CoreBuilder<K, super::resolver::mocks::MockResolver, I, O, HC>
    where
        S: FnOnce(&mut super::resolver::mocks::MockResolver),
    {
        let mut resolver = super::resolver::mocks::MockResolver::from(self.config.clone());
        setup(&mut resolver);

        CoreBuilder {
            config: self.config,
            resolver: Arc::new(resolver),
            keychain: self.keychain,
            input: self.input,
            output: self.output,
            http_connector: self.http_connector,
        }
    }

    #[cfg(test)]
    pub fn with_mock_output(self) -> CoreBuilder<K, R, I, super::output::mocks::MockOutput, HC> {
        let output = super::output::mocks::MockOutput::from(self.config.clone());

        CoreBuilder {
            config: self.config,
            resolver: self.resolver,
            keychain: self.keychain,
            input: self.input,
            output: Arc::new(output),
            http_connector: self.http_connector,
        }
    }

    #[cfg(test)]
    pub fn with_mock_input<S>(
        self,
        setup: S,
    ) -> CoreBuilder<K, R, super::input::mocks::MockInput, O, HC>
    where
        S: FnOnce(&mut super::input::mocks::MockInput),
    {
        let mut input = super::input::mocks::MockInput::from(self.config.clone());
        setup(&mut input);

        CoreBuilder {
            config: self.config,
            resolver: self.resolver,
            keychain: self.keychain,
            input: Arc::new(input),
            output: self.output,
            http_connector: self.http_connector,
        }
    }

    #[cfg(test)]
    pub fn with_http_connector<S: hyper::client::connect::Connect>(
        self,
        connector: S,
    ) -> CoreBuilder<K, R, I, O, S>
    where
        S: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {
        CoreBuilder {
            config: self.config,
            resolver: self.resolver,
            keychain: self.keychain,
            input: self.input,
            output: self.output,
            http_connector: connector,
        }
    }
}
