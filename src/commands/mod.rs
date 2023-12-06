use super::core;
use super::errors;
use super::online;
use super::tasks;
use async_trait::async_trait;
use clap::ArgMatches;
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

inventory::collect!(Command);

#[macro_export]
macro_rules! command {
    ($name:expr) => {
        inventory::submit! { Command::new(&$name) }
    };
}

#[derive(Clone)]
pub struct Command(&'static dyn CommandRunnable);

impl Command {
    pub const fn new<T>(command: &'static T) -> Self
    where
        T: CommandRunnable + 'static,
    {
        Self(command)
    }
}

#[async_trait]
impl CommandRunnable for Command {
    fn name(&self) -> String {
        self.0.name()
    }

    fn app(&self) -> clap::Command {
        self.0.app()
    }

    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        self.0.run(core, matches).await
    }

    async fn complete(&self, core: &Core, completer: &Completer, matches: &ArgMatches) {
        self.0.complete(core, completer, matches).await
    }
}

#[async_trait]
pub trait CommandRunnable: Send + Sync {
    fn name(&self) -> String;
    fn app(&self) -> clap::Command;
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error>;
    async fn complete(&self, core: &Core, completer: &Completer, matches: &ArgMatches);
}
