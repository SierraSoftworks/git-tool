mod app;
mod auth;
mod builder;
mod config;
pub mod features;
mod http;
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
use mocktopus::macros::*;

use super::errors;
pub use errors::Error;

pub use self::http::HttpClient;
pub use app::App;
pub use auth::KeyChain;
pub use config::Config;
pub use launcher::Launcher;
pub use prompt::Prompter;
pub use repo::Repo;
pub use resolver::Resolver;
pub use scratchpad::Scratchpad;
pub use service::{Service, ServiceAPI};
pub use target::Target;

#[cfg_attr(test, mockable)]
pub struct Core {
    config: Arc<Config>,
    launcher: Arc<Launcher>,
    resolver: Arc<Resolver>,
    keychain: Arc<KeyChain>,
    http_client: Arc<HttpClient>,
}

#[cfg_attr(test, mockable)]
impl Core {
    pub fn builder() -> builder::CoreBuilder {
        let config = Arc::new(Config::default());
        builder::CoreBuilder { config }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn keychain(&self) -> &KeyChain {
        &self.keychain
    }

    pub fn launcher(&self) -> &Launcher {
        &self.launcher
    }

    pub fn resolver(&self) -> &Resolver {
        &self.resolver
    }

    pub fn output(&self) -> Box<dyn Write + Send> {
        crate::console::output::output()
    }

    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }
}
