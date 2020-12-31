mod app;
mod auth;
mod config;
mod core;
mod features;
mod input;
mod launcher;
mod output;
mod prompt;
mod repo;
mod resolver;
mod scratchpad;
mod service;
mod target;
mod templates;

use super::errors;
pub use errors::Error;

pub use self::core::{Core, CoreBuilder};
pub use app::App;
pub use auth::KeyChain;
pub use config::Config;
pub use input::Input;
pub use launcher::Launcher;
pub use output::Output;
pub use prompt::prompt;
pub use repo::Repo;
pub use resolver::Resolver;
pub use scratchpad::Scratchpad;
pub use service::Service;
pub use target::Target;

pub type DefaultCore = core::DefaultCore;
pub type DefaultLauncher = launcher::TokioLauncher;
pub type DefaultResolver = resolver::FileSystemResolver;
pub type DefaultKeyChain = auth::SystemKeyChain;
pub type DefaultInput = input::StdinInput;
pub type DefaultOutput = output::StdoutOutput;
