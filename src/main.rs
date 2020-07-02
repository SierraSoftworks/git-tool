extern crate base64;
extern crate chrono;
extern crate clap;
extern crate gtmpl;
extern crate hyper;
extern crate keyring;
extern crate tokio;

use clap::{Arg, App, ArgMatches};
use std::sync::Arc;
use crate::commands::CommandRunnable;

#[macro_use] mod tasks;
mod core;
mod errors;
mod fs;
mod search;
mod git;
mod online;
mod completion;
mod commands;
mod update;

#[cfg(test)] mod test;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let commands = commands::default_commands();

    let mut app = App::new("Git-Tool")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Benjamin Pannell <benjamin@pannell.dev>")
        .about("Simplify your Git repository management and stop thinking about folders.")
        .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .env("GITTOOL_CONFIG")
                .value_name("FILE")
                .help("The path to your git-tool configuration file.")
                .takes_value(true))
        .arg(Arg::with_name("verbose")
                .long("verbose")
                .help("enable verbose logging")
                .default_value("false"))
        .arg(Arg::with_name("update-resume-internal")
            .long("update-resume-internal")
            .help("A legacy flag used to coordinate updates in the same way that the `update --state` flag is used now. Maintained for backwards compatibility reasons.")
            .takes_value(true)
            .hidden(true))
        .subcommands(commands.iter().map(|x| x.app()));

    let matches = app.clone().get_matches();

    match run(commands, matches).await {
        Result::Ok(-1) => {
            app.print_help()?;
        },
        Result::Ok(status) => {
            std::process::exit(status);
        },
        Result::Err(err) => {
            println!("{}", err.message());
            std::process::exit(1);
        },
    }

    Ok(())
}

async fn run<'a>(commands: Vec<Arc<dyn CommandRunnable<core::DefaultKeyChain, core::DefaultLauncher, core::DefaultResolver>>>, matches: ArgMatches<'a>) -> Result<i32, errors::Error> {
    let mut core_builder = core::Core::builder();

    if let Some(cfg_file) = matches.value_of("config") {
        core_builder = core_builder.with_config_file(cfg_file)?;
    }

    let core = Arc::new(core_builder.build());

    // Legacy update interoperability for compatibility with the Golang implementation
    if let Some(state) = matches.value_of("update-resume-internal") {
        if let Some(cmd) = commands.iter().find(|c| c.name() == "update") {
            let matches = cmd.app().get_matches_from(vec!["update", "--state", state]);

            return cmd.run(&core, &matches).await;
        }
    }

    for cmd in commands.iter() {
        if let Some(cmd_matches) = matches.subcommand_matches(cmd.name()) {
            return cmd.run(&core, cmd_matches).await;
        }
    }

    Ok(-1)
}