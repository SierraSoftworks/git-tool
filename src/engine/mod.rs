mod app;
mod auth;
pub mod builder;
mod config;
pub mod features;
mod http;
mod identifier;
mod launcher;
mod prompt;
mod repo;
mod resolver;
mod scratchpad;
mod service;
mod target;
mod templates;

use std::{io::Write, sync::Arc};

#[cfg(test)]
pub use self::http::mocks::MockHttpRoute;

use crate::console::ConsoleProvider;

use super::errors;
pub use errors::Error;

pub use self::http::HttpClient;
pub use app::App;
pub use auth::KeyChain;
pub use config::Config;
pub use identifier::Identifier;
pub use launcher::Launcher;
pub use prompt::Prompter;
pub use repo::Repo;
pub use resolver::Resolver;
pub use scratchpad::Scratchpad;
pub use service::{Service, ServiceAPI};
pub use target::{Target, TempTarget};

pub struct Core {
    config: Arc<Config>,
    console: Arc<dyn ConsoleProvider + Send + Sync>,
    launcher: Arc<dyn Launcher + Send + Sync>,
    resolver: Arc<dyn Resolver + Send + Sync>,
    keychain: Arc<dyn KeyChain + Send + Sync>,
    http_client: Arc<dyn HttpClient + Send + Sync>,
}

impl Core {
    pub fn builder() -> builder::CoreBuilderWithoutConfig {
        builder::CoreBuilderWithoutConfig
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn keychain(&self) -> &(dyn KeyChain + Send + Sync) {
        self.keychain.as_ref()
    }

    pub fn launcher(&self) -> &(dyn Launcher + Send + Sync) {
        self.launcher.as_ref()
    }

    pub fn resolver(&self) -> &(dyn Resolver + Send + Sync) {
        self.resolver.as_ref()
    }

    pub fn console(&self) -> Arc<dyn ConsoleProvider + Send + Sync> {
        self.console.clone()
    }

    pub fn output(&self) -> Box<dyn Write + Send> {
        self.console.output()
    }

    pub fn prompter(&self) -> Prompter {
        Prompter::new(self.console.clone())
    }

    pub fn http_client(&self) -> &(dyn HttpClient + Send + Sync) {
        self.http_client.as_ref()
    }
}
