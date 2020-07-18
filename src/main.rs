extern crate base64;
extern crate chrono;
extern crate clap;
extern crate gtmpl;
extern crate hyper;
#[cfg(test)]
#[macro_use]
extern crate yup_hyper_mock as hyper_mock;
extern crate keyring;
#[macro_use]
extern crate log;
extern crate rpassword;
extern crate sentry;
#[macro_use]
extern crate serde_json;
extern crate tokio;

use crate::commands::CommandRunnable;
use crate::core::DefaultCore;
use clap::{App, Arg, ArgMatches};
use std::sync::Arc;

#[macro_use]
mod macros;
#[macro_use]
mod tasks;
mod commands;
mod completion;
mod core;
mod errors;
mod fs;
mod git;
mod online;
mod search;
mod update;

#[cfg(test)]
mod test;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let raven = sentry::init((
        "https://0787127414b24323be5a3d34767cb9b8@o219072.ingest.sentry.io/1486938",
        sentry::ClientOptions {
            release: Some(version!("git-tool@v").into()),
            default_integrations: true,
            ..Default::default()
        }
        .add_integration(sentry::integrations::log::LogIntegration::default()),
    ));

    let commands = commands::default_commands();
    let version = version!("v");

    let mut app = App::new("Git-Tool")
        .version(version.as_str())
        .author("Benjamin Pannell <benjamin@pannell.dev>")
        .about("Simplify your Git repository management and stop thinking about where things belong.")
        .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .env("GITTOOL_CONFIG")
                .value_name("FILE")
                .help("The path to your git-tool configuration file.")
                .global(true)
                .takes_value(true))
        .arg(Arg::with_name("update-resume-internal")
            .long("update-resume-internal")
            .help("A legacy flag used to coordinate updates in the same way that the `update --state` flag is used now. Maintained for backwards compatibility reasons.")
            .takes_value(true)
            .hidden(true))
        .subcommands(commands.iter().map(|x| x.app()));

    let matches = app.clone().get_matches();

    match run(&mut app, commands, matches).await {
        Result::Ok(status) => {
            raven.close(None);
            std::process::exit(status);
        }
        Result::Err(err) => {
            error!("{}", err.message());
            println!("{}", err.message());

            if err.is_system() {
                sentry::capture_error(&err);
            }

            raven.close(None);
            std::process::exit(1);
        }
    }
}

async fn run<'a, 'b: 'a>(
    app: &'a mut App<'a, 'b>,
    commands: Vec<Arc<dyn CommandRunnable<DefaultCore>>>,
    matches: ArgMatches<'b>,
) -> Result<i32, errors::Error> {
    let mut core_builder = core::CoreBuilder::default();

    if let Some(cfg_file) = matches.value_of("config") {
        core_builder = core_builder.with_config_file(cfg_file)?;
    }

    let core = Arc::new(core_builder.build());

    // Legacy update interoperability for compatibility with the Golang implementation
    if let Some(state) = matches.value_of("update-resume-internal") {
        if let Some(cmd) = commands.iter().find(|c| c.name() == "update") {
            let matches = cmd
                .app()
                .get_matches_from_safe(vec!["gt", "update", "--state", &state])
                .map_err(|e| errors::system_with_internal("Failed to process internal update operation.",
                    "Please report this error to us on GitHub and use the manual update process until it is resolved.",
                    e))?;

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

    warn!("You have not provided a valid command or command alias. Try running `git-tool open github.com/your/repo`.");
    app.print_help().unwrap_or_default();

    Ok(-1)
}
