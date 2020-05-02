extern crate clap;
extern crate gtmpl;
extern crate hyper;
extern crate tokio;
extern crate chrono;

use clap::{Arg, App, ArgMatches};
use std::sync::Arc;

mod commands;
mod core;
mod errors;
mod online;
mod search;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = App::new("Git-Tool")
        .version("2.0.0")
        .author("Benjamin Pannell <benjamin@pannell.dev>")
        .about("")
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
        .subcommands(commands::commands().into_iter().map(|x| x.app()));

    let matches = app.get_matches();

    match run(matches).await {
        Result::Ok(status) => {
            std::process::exit(status);
        },
        Result::Err(err) => {
            println!("{}", err.message());
            std::process::exit(1);
        },
    }
}

async fn run<'a>(matches: ArgMatches<'a>) -> Result<i32, errors::Error> {
    let mut core_builder = core::Core::builder();

    if let Some(cfg_file) = matches.value_of("config") {
        core_builder.with_config_file(cfg_file)?;
    }

    let core = Arc::new(core_builder.build());

    for cmd in commands::commands().iter() {
        if let Some(cmd_matches) = matches.subcommand_matches(cmd.name()) {
            let status = cmd.run(core.clone(), cmd_matches).await?;
            return Ok(status);
        }
    }

    return Err(errors::user(
        "We did not recognize the command you provided.",
        "Please run `git-tool --help` and ensure you are using a supported command."))
}