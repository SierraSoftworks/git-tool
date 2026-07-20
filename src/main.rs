#![allow(clippy::blocks_in_conditions)]

extern crate base64;
extern crate chrono;
extern crate clap;
extern crate gotmpl;
extern crate lazy_static;
extern crate serde_json;
extern crate tokio;
extern crate tracing_batteries;

use git_tool::{
    commands::{self, CommandRunnable},
    engine::{self, features},
    safe_eprintln, telemetry,
};
use std::sync::{Arc, atomic::AtomicBool};
use tracing_batteries::{Session, prelude::*};

#[cfg(feature = "telemetry")]
type TelemetrySession = Arc<Session>;
#[cfg(not(feature = "telemetry"))]
type TelemetrySession = ();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::process::exit({
        let session_id = std::env::var("GITTOOL_SESSION_ID")
            .unwrap_or_else(|_| tracing_batteries::Analytics::session_id());

        let session = telemetry::setup(&session_id);

        let app = commands::app();

        match host(app, &session, &session_id).await {
            Ok(status) => {
                telemetry::shutdown(session);
                status
            }
            Err(err) => {
                if err.is(human_errors::Kind::System) {
                    #[cfg(feature = "telemetry")]
                    session.record_human_error(&err);
                }

                telemetry::shutdown(session);
                1
            }
        }
    });
}

#[tracing::instrument(err, skip(app, session), fields(otel.name=EmptyField, command=EmptyField, exit_code=EmptyField, otel.status_code=0, exception=EmptyField))]
async fn host(
    app: clap::Command,
    session: &TelemetrySession,
    session_id: &str,
) -> Result<i32, human_errors::Error> {
    let matches = match app.clone().try_get_matches() {
        Ok(matches) => {
            if let Some(context) = matches.get_one::<String>("trace-context") {
                load_trace_context(&Span::current(), context);
                debug!("Loaded trace context from command line parameters.");
            }

            let command_name = format!("gt {}", matches.subcommand_name().unwrap_or(""))
                .trim()
                .to_string();

            Span::current().record("otel.name", command_name.as_str());

            matches
        }
        Err(error)
            if error.kind() != clap::error::ErrorKind::DisplayVersion
                && error.kind() != clap::error::ErrorKind::DisplayHelp =>
        {
            Span::current()
                .record("otel.status_code", 2_u32)
                .record("exit_code", 1_u32)
                .record("exception", display(&error));

            #[cfg(feature = "telemetry")]
            if session.enable().load(std::sync::atomic::Ordering::Relaxed) {
                safe_eprintln!(
                    "Trace ID: {:032x}",
                    Span::current().context().span().span_context().trace_id()
                );
            }

            error.print().unwrap_or_default();

            return Err(human_errors::wrap_user(
                error,
                "You did no provide a valid set of command line arguments to Git-Tool.",
                &[
                    "Read the help message printed above and try running Git-Tool again, or take a look at our documentation on https://git-tool.sierrasoftworks.com.",
                ],
            ));
        }
        Err(error) => {
            error.print().unwrap_or_default();

            Span::current()
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

    let subcommand_name = matches.subcommand_name().unwrap_or_default();
    let command_name = format!("gt {}", subcommand_name).trim().to_string();

    #[cfg(feature = "telemetry")]
    let telemetry_enabled = session.enable();
    #[cfg(not(feature = "telemetry"))]
    let telemetry_enabled = Arc::new(AtomicBool::new(false));

    // Disable telemetry emission for the `shell-init` and `complete` subcommands, since they are invoked by the shell
    // and not by the user directly (i.e. they mis-represent user engagement with the tool and lead to overly-chatty
    // telemetry data).
    if !matches.get_flag("trace")
        && (subcommand_name == "shell-init" || subcommand_name == "complete")
    {
        telemetry_enabled.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    Span::current()
        .record("command", command_name.as_str())
        .record("otel.name", command_name.as_str());

    #[cfg(feature = "telemetry")]
    let _page = session.record_new_page(format!("/{}", subcommand_name));

    #[cfg(feature = "telemetry")]
    let analytics = engine::Analytics::new(session.clone(), session_id);
    #[cfg(not(feature = "telemetry"))]
    let analytics = engine::Analytics::disabled();

    // The event is named after the command so that a session trace reads
    // naturally; the name is one of the hard-coded subcommands (clap rejects
    // anything else before we get here) or `help` when none was given.
    let command_event = format!("commands::{}", matches.subcommand_name().unwrap_or("help"));

    match run(matches, telemetry_enabled.clone(), analytics.clone()).await {
        Ok(-2) => {
            app.clone().print_help().unwrap_or_default();

            Span::current()
                .record("otel.status_code", 2_u32)
                .record("exit_code", 2_u32);

            analytics.record_event("commands::help", [("subcommand", command_name.clone())]);

            warn!("Exiting with status code {}", 2);
            Ok(2)
        }
        Ok(status) => {
            info!("Exiting with status code {}", status);
            Span::current().record("exit_code", status);

            Ok(status)
        }
        Err(error) => {
            safe_eprintln!("{}", human_errors::pretty(&error));

            error!("Exiting with status code {}", 1);
            Span::current()
                .record("otel.status_code", 2_u32)
                .record("exit_code", 1_u32);

            if error.is(human_errors::Kind::System) {
                Span::current().record("exception", display(&error));
                let error = tracing_batteries::ErrorInfo::from_human_error(&error).with_metadata(
                    "trace.id",
                    format!(
                        "{:032x}",
                        Span::current().context().span().span_context().trace_id()
                    ),
                );

                analytics.record_custom_error(error);
            } else {
                Span::current().record("exception", error.description());

                analytics.record_event(
                    command_event.clone(),
                    [
                        ("status", "failed".to_string()),
                        (
                            "error_kind",
                            if error.is(human_errors::Kind::System) {
                                "system".to_string()
                            } else {
                                "user".to_string()
                            },
                        ),
                    ],
                );
            }

            if telemetry_enabled.load(std::sync::atomic::Ordering::Relaxed) {
                safe_eprintln!(
                    "Trace ID: {:032x}",
                    Span::current().context().span().span_context().trace_id()
                );
            }

            Err(error)
        }
    }
}

#[tracing::instrument(err, skip(matches, analytics), fields(command=matches.subcommand_name().unwrap_or("")))]
async fn run(
    matches: clap::ArgMatches,
    telemetry_enabled: Arc<AtomicBool>,
    analytics: engine::Analytics,
) -> Result<i32, human_errors::Error> {
    let core_builder = engine::Core::builder();

    let core_builder = if let Some(cfg_file) = matches.get_one::<String>("config") {
        debug!("Loading configuration file...");
        core_builder.with_config_file(cfg_file)?
    } else if let Some(dirs) = engine::Config::default_path() {
        debug!("Loading configuration from default config file...");
        core_builder.with_config_file_or_default(dirs)
    } else {
        warn!(
            "No configuration file was specified and we were unable to determine the default configuration file location."
        );
        core_builder.with_default_config()
    };

    let core = core_builder.with_analytics(analytics).build();

    // If telemetry is enabled in the config file, then turn it on here.
    if !core.config().get_features().has(features::TELEMETRY) {
        telemetry_enabled.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    // If the user explicitly enables tracing, then turn it on and print your trace ID
    if matches.get_flag("trace") {
        debug!("Tracing enabled by command line flag.");
        telemetry_enabled.store(true, std::sync::atomic::Ordering::Relaxed);
        safe_eprintln!(
            "Tracing enabled, your trace ID is: {:032x}",
            Span::current().context().span().span_context().trace_id()
        );
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

fn load_trace_context(span: &Span, context: &str) {
    let carrier: std::collections::HashMap<String, String> =
        serde_json::from_str(context).unwrap_or_default();
    let parent_context = opentelemetry::global::get_text_map_propagator(|p| p.extract(&carrier));
    span.set_parent(parent_context).unwrap_or_default();
}
