use super::core;
use super::errors;
use super::online;
use super::tasks;
use async_trait::async_trait;
use clap::ArgMatches;
use std::sync::Arc;
use std::{io::Write, vec::Vec};

use crate::{completion::Completer, core::Core};

mod apps;

#[cfg(feature = "auth")]
mod auth;
mod clone;
mod complete;
mod config;
mod doctor;
mod fix;
mod helpers;
mod ignore;
mod info;
mod list;
mod new;
mod open;
mod prune;
mod remove;
mod scratch;
mod services;
mod setup;
mod shell_init;
mod switch;
mod update;

pub trait Command: Send + Sync {
    fn name(&self) -> String;
    fn app<'a>(&self) -> clap::Command<'a>;
}

#[async_trait]
pub trait CommandRunnable: Command {
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error>;
    async fn complete(&self, core: &Core, completer: &Completer, matches: &ArgMatches);
}

pub fn default_commands() -> Vec<Arc<dyn CommandRunnable>> {
    commands()
}

pub fn commands() -> Vec<Arc<dyn CommandRunnable>> {
    vec![
        Arc::new(apps::AppsCommand {}),
        #[cfg(feature = "auth")]
        Arc::new(auth::AuthCommand {}),
        Arc::new(clone::CloneCommand {}),
        Arc::new(complete::CompleteCommand {}),
        Arc::new(config::ConfigCommand {}),
        Arc::new(doctor::DoctorCommand {}),
        Arc::new(fix::FixCommand {}),
        Arc::new(info::InfoCommand {}),
        Arc::new(ignore::IgnoreCommand {}),
        Arc::new(list::ListCommand {}),
        Arc::new(new::NewCommand {}),
        Arc::new(open::OpenCommand {}),
        Arc::new(prune::PruneCommand {}),
        Arc::new(remove::RemoveCommand {}),
        Arc::new(scratch::ScratchCommand {}),
        Arc::new(services::ServicesCommand {}),
        Arc::new(setup::SetupCommand {}),
        Arc::new(shell_init::ShellInitCommand {}),
        Arc::new(update::UpdateCommand {}),
        Arc::new(switch::SwitchCommand {}),
    ]
}
