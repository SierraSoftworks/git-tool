#![allow(clippy::blocks_in_conditions)]

extern crate base64;
extern crate chrono;
extern crate clap;
extern crate gtmpl;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate tracing;
extern crate sentry;
#[macro_use]
extern crate serde_json;
extern crate tokio;

use crate::commands::CommandRunnable;
use crate::core::features;
use clap::{crate_authors, Arg};
use opentelemetry::{propagation::TextMapPropagator, trace::TraceContextExt};
use telemetry::Session;
use tracing::field;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[macro_use]
mod macros;
#[macro_use]
mod tasks;
#[macro_use]
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
    std::process::exit({
        let session = Session::new();

        let app = build_app();

        match host(app).await {
            Result::Ok(status) => {
                session.complete();
                status
            }
            Result::Err(err) => {
                session.crash(err);
                1
            }
        }
    });
}

#[allow(non_upper_case_globals)]
fn build_app() -> clap::Command {
    clap::Command::new("Git-Tool")
            .version(version!("v"))
            .author(crate_authors!("\n"))
            .about("Simplify your Git repository management and stop thinking about where things belong.")
            .arg(Arg::new("config")
                .short('c')
                .long("config")
                .env("GITTOOL_CONFIG")
                .value_name("FILE")
                .help("The path to your git-tool configuration file.")
                .action(clap::ArgAction::Set))
            .arg(Arg::new("update-resume-internal")
                .long("update-resume-internal")
                .help("A legacy flag used to coordinate updates in the same way that the `update --state` flag is used now. Maintained for backwards compatibility reasons.")
                .action(clap::ArgAction::Set)
                .hide(true))
            .arg(Arg::new("trace")
                .long("trace")
                .global(true)
                .help("Enable tracing for the current command and print the trace ID to assist with bug reports."))
            .arg(Arg::new("trace-context")
                .long("trace-context")
                .help("Configures the trace context used by this Git-Tool execution.")
                .action(clap::ArgAction::Set))
            .subcommands(inventory::iter::<commands::Command>().map(|x| x.app()))
}

#[tracing::instrument(err, skip(app), fields(otel.name=field::Empty, command=field::Empty, exit_code=field::Empty, otel.status_code=0, exception=field::Empty))]
async fn host(app: clap::Command) -> Result<i32, errors::Error> {
    let matches = match app.clone().try_get_matches() {
        Ok(matches) => {
            if let Some(context) = matches.get_one::<String>("trace-context") {
                load_trace_context(&tracing::Span::current(), context);
                info!("Loaded trace context from command line parameters.");
            }

            let command_name = format!("gt {}", matches.subcommand_name().unwrap_or(""))
                .trim()
                .to_string();

            tracing::Span::current().record("otel.name", command_name.as_str());

            matches
        }
        Err(error)
            if error.kind() != clap::error::ErrorKind::DisplayVersion
                && error.kind() != clap::error::ErrorKind::DisplayHelp =>
        {
            tracing::Span::current()
                .record("otel.status_code", 2_u32)
                .record("exit_code", 1_u32)
                .record("exception", &field::display(&error));

            if telemetry::is_enabled() {
                println!(
                    "Trace ID: {:032x}",
                    tracing::Span::current()
                        .context()
                        .span()
                        .span_context()
                        .trace_id()
                );
            }

            error.print().unwrap_or_default();

            return Err(errors::user_with_internal(
                "You did no provide a valid set of command line arguments to Git-Tool.",
                "Read the help message printed above and try running Git-Tool again, or take a look at our documentation on https://git-tool.sierrasoftworks.com.",
                error,
            ));
        }
        Err(error) => {
            error.print().unwrap_or_default();

            tracing::Span::current()
                .record(
                    "otel.name",
                    if error.kind() == clap::error::ErrorKind::DisplayVersion {
                        "gt --version"
                    } else {
                        "gt --help"
                    },
                )
                .record("exit_code", 2_u32);

            return Ok(2);
        }
    };

    let command_name = format!("gt {}", matches.subcommand_name().unwrap_or(""))
        .trim()
        .to_string();

    tracing::Span::current()
        .record("command", command_name.as_str())
        .record("otel.name", command_name.as_str());

    match run(matches).await {
        Ok(-2) => {
            app.clone().print_help().unwrap_or_default();

            tracing::Span::current()
                .record("otel.status_code", 2_u32)
                .record("exit_code", 2_u32);

            warn!("Exiting with status code {}", 2);
            Ok(2)
        }
        Ok(status) => {
            info!("Exiting with status code {}", status);
            tracing::Span::current().record("exit_code", status);
            Ok(status)
        }
        Err(error) => {
            println!("{error}");

            error!("Exiting with status code {}", 1);
            tracing::Span::current()
                .record("otel.status_code", 2_u32)
                .record("exit_code", 1_u32);

            if error.is_system() {
                tracing::Span::current().record("exception", &field::display(&error));
            } else {
                tracing::Span::current().record("exception", &error.description());
            }

            if telemetry::is_enabled() {
                println!(
                    "Trace ID: {:032x}",
                    tracing::Span::current()
                        .context()
                        .span()
                        .span_context()
                        .trace_id()
                );
            }

            Err(error)
        }
    }
}

#[tracing::instrument(err, skip(matches), fields(command=matches.subcommand_name().unwrap_or("")))]
async fn run(matches: clap::ArgMatches) -> Result<i32, errors::Error> {
    let core_builder = core::Core::builder();

    let core_builder = if let Some(cfg_file) = matches.get_one::<String>("config") {
        debug!("Loading configuration file...");
        core_builder.with_config_file(cfg_file)?
    } else if let Some(dirs) = core::Config::default_path() {
        debug!("Loading configuration from default config file...");
        core_builder.with_config_file_or_default(dirs)
    } else {
        warn!("No configuration file was specified and we were unable to determine the default configuration file location.");
        core_builder.with_default_config()
    };

    let core = core_builder.build();

    // If telemetry is enabled in the config file, then turn it on here.
    if !core.config().get_features().has(features::TELEMETRY) {
        telemetry::set_enabled(false);
    }

    // If the user explicitly enables tracing, then turn it on and print your trace ID
    if matches.contains_id("trace") {
        debug!("Tracing enabled by command line flag.");
        telemetry::set_enabled(true);
        writeln!(
            core.output(),
            "Tracing enabled, your trace ID is: {:032x}",
            tracing::Span::current()
                .context()
                .span()
                .span_context()
                .trace_id()
        )?;
    }

    // Legacy update interoperability for compatibility with the Golang implementation
    if let (Some(state), None) = (
        matches.get_one::<String>("update-resume-internal"),
        matches.subcommand_name(),
    ) {
        info!("Detected the legacy --update-resume-internal flag, rewriting it to use the new update sub-command.");
        if let Some(cmd) = inventory::iter::<commands::Command>().find(|c| c.name() == "update") {
            let matches = cmd
                .app()
                .try_get_matches_from(vec!["gt", "update", "--state", state])
                .map_err(|e| errors::system_with_internal("Failed to process internal update operation.",
                    "Please report this error to us on GitHub and use the manual update process until it is resolved.",
                    errors::detailed_message(&e.to_string())))?;

            info!(
                "Running update sub-command with state sourced from --update-resume-internal flag."
            );
            return cmd.run(&core, &matches).await;
        }
    }

    debug!("Looking for an appropriate matching command implementation.");
    for cmd in inventory::iter::<commands::Command> {
        if let Some(cmd_matches) = matches.subcommand_matches(&cmd.name()) {
            debug!("Found a command implementation for '{}'", cmd.name());

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

    warn!("Did not find a matching command, printing the help message.");
    Ok(-2)
}

fn load_trace_context(span: &tracing::Span, context: &str) {
    let carrier: std::collections::HashMap<String, String> =
        serde_json::from_str(context).unwrap_or_default();
    let propagator = opentelemetry_sdk::propagation::TraceContextPropagator::new();
    let parent_context = propagator.extract(&carrier);
    span.set_parent(parent_context);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_command() {
        let app = build_app();
        app.debug_assert();
    }
}
