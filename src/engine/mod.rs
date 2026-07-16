mod analytics;
mod app;
mod auth;
mod branch;
pub mod builder;
mod config;
pub mod features;
mod http;
mod identifier;
mod launcher;
mod prompt;
mod repo;
mod repo_config;
mod resolve;
mod scratchpad;
mod service;
mod target;
mod templates;
mod worktree;

use std::{io::Write, sync::Arc};

#[cfg(test)]
pub use self::http::mocks::MockHttpRoute;

use crate::console::ConsoleProvider;

pub use human_errors::Error;

pub use self::http::HttpClient;
pub use analytics::Analytics;
pub use app::App;
pub use auth::KeyChain;
pub use branch::Branch;
pub use config::Config;
pub use identifier::Identifier;
pub use launcher::Launcher;
pub use prompt::Prompter;
pub use repo::Repo;
pub use repo_config::RepoConfig;
use resolve::ResolverBackend;
pub use resolve::{ResolveMany, Resolver};
pub use scratchpad::Scratchpad;
pub use service::{Service, ServiceAPI};
pub use target::{Target, TempMode, TempTarget};
pub use templates::{render, render_list};
pub use worktree::Worktree;

pub struct Core {
    config: Arc<Config>,
    console: Arc<dyn ConsoleProvider + Send + Sync>,
    launcher: Arc<dyn Launcher + Send + Sync>,
    resolver: ResolverBackend,
    keychain: Arc<dyn KeyChain + Send + Sync>,
    http_client: Arc<dyn HttpClient + Send + Sync>,
    analytics: Analytics,
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

    pub fn analytics(&self) -> &Analytics {
        &self.analytics
    }
}
