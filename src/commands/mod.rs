use super::core;
use super::errors;
use super::online;
use super::tasks;
use async_trait::async_trait;
use clap::{App, ArgMatches};
use std::sync::Arc;
use std::vec::Vec;

use crate::{
    completion::Completer,
    core::{
        Core, DefaultFileSource, DefaultLauncher, DefaultResolver, FileSource, Launcher, Resolver,
    },
};

mod apps;
mod branch;
mod complete;
mod config;
mod ignore;
mod info;
mod list;
mod new;
mod open;
mod scratch;
mod services;
mod shell_init;
mod update;

pub trait Command: Send + Sync {
    fn name(&self) -> String;
    fn app<'a, 'b>(&self) -> App<'a, 'b>;
}

#[async_trait]
pub trait CommandRunnable<
    F: FileSource = DefaultFileSource,
    L: Launcher = DefaultLauncher,
    R: Resolver = DefaultResolver,
>: Command
{
    async fn run<'a>(
        &self,
        core: &Core<F, L, R>,
        matches: &ArgMatches<'a>,
    ) -> Result<i32, errors::Error>;
    async fn complete<'a>(
        &self,
        core: &Core<F, L, R>,
        completer: &Completer,
        matches: &ArgMatches<'a>,
    );
}

pub fn default_commands(
) -> Vec<Arc<dyn CommandRunnable<DefaultFileSource, DefaultLauncher, DefaultResolver>>> {
    commands()
}

pub fn commands<F: FileSource, L: Launcher, R: Resolver>() -> Vec<Arc<dyn CommandRunnable<F, L, R>>>
{
    vec![
        Arc::new(apps::AppsCommand{}),
        Arc::new(branch::BranchCommand {}),
        Arc::new(complete::CompleteCommand{}),
        Arc::new(config::ConfigCommand{}),
        Arc::new(info::InfoCommand{}),
        Arc::new(ignore::IgnoreCommand{}),
        Arc::new(list::ListCommand{}),
        Arc::new(new::NewCommand{}),
        Arc::new(open::OpenCommand{}),
        Arc::new(scratch::ScratchCommand{}),
        Arc::new(services::ServicesCommand{}),
        Arc::new(shell_init::ShellInitCommand{}),
        Arc::new(update::UpdateCommand{}),
    ]
}
