use super::core;
use super::errors;
use super::online;
use super::tasks;
use async_trait::async_trait;
use clap::{App, ArgMatches};
use std::sync::Arc;
use std::{io::Write, vec::Vec};

use crate::{
    completion::Completer,
    core::{Core, DefaultCore, KeyChain, Launcher, Output, Resolver},
};

mod apps;
mod auth;
mod branch;
mod complete;
mod config;
mod helpers;
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
pub trait CommandRunnable<C: Core>: Command {
    async fn run<'a>(&self, core: &C, matches: &ArgMatches<'a>) -> Result<i32, errors::Error>;
    async fn complete<'a>(&self, core: &C, completer: &Completer, matches: &ArgMatches<'a>);
}

pub fn default_commands() -> Vec<Arc<dyn CommandRunnable<DefaultCore>>> {
    commands()
}

pub fn commands<C: Core>() -> Vec<Arc<dyn CommandRunnable<C>>> {
    vec![
        Arc::new(apps::AppsCommand {}),
        Arc::new(auth::AuthCommand {}),
        Arc::new(branch::BranchCommand {}),
        Arc::new(complete::CompleteCommand {}),
        Arc::new(config::ConfigCommand {}),
        Arc::new(info::InfoCommand {}),
        Arc::new(ignore::IgnoreCommand {}),
        Arc::new(list::ListCommand {}),
        Arc::new(new::NewCommand {}),
        Arc::new(open::OpenCommand {}),
        Arc::new(scratch::ScratchCommand {}),
        Arc::new(services::ServicesCommand {}),
        Arc::new(shell_init::ShellInitCommand {}),
        Arc::new(update::UpdateCommand {}),
    ]
}
