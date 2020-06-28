use std::sync::Arc;
use clap::{App, ArgMatches};
use std::vec::Vec;
use super::core;
use super::tasks;
use super::errors;
use super::online;
use async_trait::async_trait;

use crate::{completion::Completer, core::{Core, Launcher, Resolver, FileSource, DefaultFileSource, DefaultLauncher, DefaultResolver}};

mod apps;
mod complete;
mod config;
mod ignore;
mod info;
mod list;
mod new;
mod open;
mod scratch;
mod services;

pub trait Command: Send + Sync {
    fn name(&self) -> String;
    fn app<'a, 'b>(&self) -> App<'a, 'b>;
}

#[async_trait]
pub trait CommandRunnable<F: FileSource = DefaultFileSource, L: Launcher = DefaultLauncher, R: Resolver = DefaultResolver> : Command {
    async fn run<'a>(&self, core: &Core<F, L, R>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error>;
    async fn complete<'a>(&self, core: &Core<F, L, R>, completer: &Completer, matches: &ArgMatches<'a>);
}

pub fn default_commands()  -> Vec<Arc<dyn CommandRunnable<DefaultFileSource, DefaultLauncher, DefaultResolver>>> {
    commands()
}

pub fn commands<F: FileSource, L: Launcher, R: Resolver>() -> Vec<Arc<dyn CommandRunnable<F, L, R>>> {
    vec![
        Arc::new(apps::AppsCommand{}),
        Arc::new(complete::CompleteCommand{}),
        Arc::new(config::ConfigCommand{}),
        Arc::new(info::InfoCommand{}),
        Arc::new(ignore::IgnoreCommand{}),
        Arc::new(list::ListCommand{}),
        Arc::new(new::NewCommand{}),
        Arc::new(open::OpenCommand{}),
        Arc::new(scratch::ScratchCommand{}),
        Arc::new(services::ServicesCommand{}),
    ]
}