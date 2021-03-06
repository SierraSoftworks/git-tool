extern crate base64;
extern crate chrono;
extern crate clap;
extern crate gtmpl;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate sentry;
#[macro_use]
extern crate serde_json;
extern crate tokio;

use crate::commands::CommandRunnable;
use crate::core::features;
use clap::{crate_authors, App, Arg, ArgMatches};
use std::sync::Arc;
use telemetry::Session;

#[macro_use]
mod macros;
#[macro_use]
mod tasks;
mod commands;
mod completion;
mod console;
mod core;
mod errors;
mod fs;
mod git;
mod online;
mod search;
mod telemetry;
mod update;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let session = Session::new();

    let commands = commands::default_commands();
    let version = version!("v");

    let app = App::new("Git-Tool")
        .version(version.as_str())
        .author(crate_authors!("\n"))
        .about("Simplify your Git repository management and stop thinking about where things belong.")
        .arg(Arg::new("config")
                .short('c')
                .long("config")
                .env("GITTOOL_CONFIG")
                .value_name("FILE")
                .about("The path to your git-tool configuration file.")
                .takes_value(true))
        .arg(Arg::new("update-resume-internal")
            .long("update-resume-internal")
            .about("A legacy flag used to coordinate updates in the same way that the `update --state` flag is used now. Maintained for backwards compatibility reasons.")
            .takes_value(true)
            .hidden(true))
        .subcommands(commands.iter().map(|x| x.app()));

    let matches = app.clone().get_matches();

    match run(app, commands, matches).await {
        Result::Ok(status) => {
            session.complete();
            std::process::exit(status);
        }
        Result::Err(err) => {
            println!("{}", err.message());

            session.crash(err);
            std::process::exit(1);
        }
    }
}

async fn run<'a>(
    mut app: App<'a>,
    commands: Vec<Arc<dyn CommandRunnable>>,
    matches: ArgMatches,
) -> Result<i32, errors::Error> {
    let mut core_builder = core::Core::builder();

    if let Some(cfg_file) = matches.value_of("config") {
        debug!("Loading configuration file.");
        core_builder = core_builder.with_config_file(cfg_file)?;
    }

    let core = Arc::new(core_builder.build());

    // If telemetry is disabled in the config file, then turn it off here.
    if !core.config().get_features().has(features::TELEMETRY) {
        telemetry::set_enabled(false);
    }

    // Legacy update interoperability for compatibility with the Golang implementation
    if let Some(state) = matches.value_of("update-resume-internal") {
        info!("Detected the legacy --update-resume-internal flag, rewriting it to use the new update sub-command.");
        if let Some(cmd) = commands.iter().find(|c| c.name() == "update") {
            let matches = cmd
                .app()
                .try_get_matches_from(vec!["gt", "update", "--state", &state])
                .map_err(|e| errors::system_with_internal("Failed to process internal update operation.",
                    "Please report this error to us on GitHub and use the manual update process until it is resolved.",
                    errors::detailed_message(&e.to_string())))?;

            info!(
                "Running update sub-command with state sourced from --update-resume-internal flag."
            );
            return cmd.run(&core, &matches).await;
        }
    }

    for cmd in commands.iter() {
        if let Some(cmd_matches) = matches.subcommand_matches(cmd.name()) {
            sentry::add_breadcrumb(sentry::Breadcrumb {
                ty: "default".into(),
                level: sentry::Level::Info,
                category: Some("commands".into()),
                message: Some(format!("Running {}", cmd.name())),
                ..Default::default()
            });

            sentry::configure_scope(|scope| {
                scope.set_transaction(Some(&cmd.name()));
                scope.set_tag("command", cmd.name());
            });

            return cmd.run(&core, cmd_matches).await;
        }
    }

    app.print_help().unwrap_or_default();
    Ok(-1)
}
