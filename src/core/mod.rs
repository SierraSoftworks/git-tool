mod app;
mod auth;
mod core;
mod config;
mod service;
mod features;
mod launcher;
mod repo;
mod resolver;
mod scratchpad;
mod target;
mod templates;
mod output;

use super::errors;
pub use errors::Error;

pub use self::core::Core;
pub use app::App;
pub use config::Config;
pub use launcher::Launcher;
pub use repo::Repo;
pub use service::Service;
pub use scratchpad::Scratchpad;
pub use target::Target;
pub use resolver::Resolver;
pub use auth::KeyChain;
pub use output::Output;

pub type DefaultLauncher = launcher::TokioLauncher;
pub type DefaultResolver = resolver::FileSystemResolver;
pub type DefaultKeyChain = auth::SystemKeyChain;
pub type DefaultOutput = output::StdoutOutput;
