use super::engine;
use super::online;
use super::tasks;
use async_trait::async_trait;
use clap::{Arg, ArgAction, ArgMatches, crate_authors};
use std::{io::Write, vec::Vec};

use crate::{
    completion::Completer,
    engine::{Core, Repo, ResolveMany, Resolver},
};

mod apps;

#[cfg(feature = "auth")]
mod auth;
mod clone;
mod complete;
mod config;
mod doctor;
mod fix;
mod ignore;
mod info;
mod list;
mod new;
mod open;
mod prune;
mod remove;
mod rename;
mod scratch;
mod services;
mod setup;
mod shell_init;
mod switch;
mod task;
mod temp;
mod trust;
mod update;
mod worktree;
inventory::collect!(Command);

#[allow(non_upper_case_globals)]
pub fn app() -> clap::Command {
    clap::Command::new("Git-Tool")
        .version(version!("v"))
        .author(crate_authors!("\n"))
        .about("Simplify your Git repository management and stop thinking about where things belong.")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .env("GITTOOL_CONFIG")
                .value_name("FILE")
                .help("The path to your git-tool configuration file.")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("update-resume-internal")
                .long("update-resume-internal")
                .help("A legacy flag emitted by older Git-Tool releases when coordinating an update. Tolerated (and ignored) so an update started by an older release can hand off to this one via the `update --state` sub-command it also passes.")
                .action(ArgAction::Set)
                .hide(true),
        )
        .arg(
            Arg::new("trace")
                .long("trace")
                .global(true)
                .help("Enable tracing for the current command and print the trace ID to assist with bug reports.")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("trace-context")
                .long("trace-context")
                .help("Configures the trace context used by this Git-Tool execution.")
                .action(ArgAction::Set),
        )
        .subcommands(inventory::iter::<Command>().map(|command| command.app()))
}

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

    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
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
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error>;
    async fn complete(&self, core: &Core, completer: &Completer, matches: &ArgMatches);

    #[cfg(test)]
    async fn assert_run_successful(&self, core: &Core, matches: &ArgMatches) {
        match self.run(core, matches).await {
            Ok(status) => {
                assert_eq!(status, 0);
            }
            Err(err) => panic!("{}", err.message()),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_graph_is_valid() {
        super::app().debug_assert();
    }
}
