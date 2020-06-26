use std::sync::Arc;
use clap::{App, ArgMatches};
use std::vec::Vec;
use super::core;
use super::tasks;
use super::errors;
use super::online;
use async_trait::async_trait;

use crate::core::{Core, Launcher, Resolver, FileSource, DefaultFileSource, DefaultLauncher, DefaultResolver};

mod apps;
mod ignore;
mod info;
mod new;
mod open;
mod scratch;
mod services;

pub trait Command {
    fn name(&self) -> String;
    fn app<'a, 'b>(&self) -> App<'a, 'b>;
}

#[async_trait]
pub trait CommandRun<F: FileSource = DefaultFileSource, L: Launcher = DefaultLauncher, R: Resolver = DefaultResolver> : Command {
    async fn run<'a>(&self, core: &Core<F, L, R>, matches: &ArgMatches<'a>) -> Result<i32, errors::Error>;
}

pub fn commands() -> Vec<Arc<dyn CommandRun<DefaultFileSource, DefaultLauncher, DefaultResolver>>> {
    vec![
        Arc::new(apps::AppsCommand{}),
        Arc::new(info::InfoCommand{}),
        Arc::new(ignore::IgnoreCommand{}),
        Arc::new(open::OpenCommand{}),
        Arc::new(scratch::ScratchCommand{}),
        Arc::new(services::ServicesCommand{}),
    ]
}