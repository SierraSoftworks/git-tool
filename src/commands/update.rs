use super::*;
use crate::{errors::HumanErrorResultExt, update::Release};
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct UpdateCommand;
crate::command!(UpdateCommand);

#[async_trait]
impl CommandRunnable for UpdateCommand {
    fn name(&self) -> String {
        String::from("update")
    }
    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("updates Git-Tool automatically by fetching the latest release from GitHub")
            .long_about("Allows you to update Git-Tool to the latest version, or a specific version, automatically.")
            .arg(Arg::new("state")
                .long("state")
                .help("Serialized state used to resume an in-progress update. Set automatically when the updater relaunches Git-Tool between phases.")
                .hide(true)
                .action(clap::ArgAction::Set))
            .arg(Arg::new("list")
                .long("list")
                .help("Prints the list of available releases.")
                .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("target-version")
                .help("The version you wish to update to. Defaults to the latest available version.")
                .index(1))
            .arg(Arg::new("prerelease")
                .help("Install pre-release and/or early access versions of Git-Tool.")
                .long("prerelease")
                .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt update", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, engine::Error> {
        let manager = crate::update::manager();

        // When the updater relaunches us between phases it invokes
        // `gt update --state <json>`; hand that straight back to the updater to
        // continue the in-progress update (the state also carries the trace
        // context, so the phases stay on one distributed trace).
        if let Some(state) = matches.get_one::<String>("state") {
            info!("Resuming an in-progress update.");
            manager.resume_from_arg(state).await?;
            return Ok(0);
        }

        let current_version: semver::Version = version!().parse().map_err(|err| human_errors::wrap_system(
                err,
                "Could not parse the current application version into a SemVer version number.",
                &["Please report this issue to us on GitHub and try updating manually by downloading the latest release from GitHub once the problem is resolved."],
            ))?;

        let releases = manager.get_releases().await?;

        if matches.get_flag("list") {
            write!(
                core.output(),
                "{}",
                format_release_list(&releases, &current_version)
            )
            .to_human_error()?;

            return Ok(0);
        }

        let include_prerelease = matches.get_flag("prerelease");
        let requested_version = matches.get_one::<String>("target-version");

        let target_release = match requested_version {
            // An explicit version (rollback or roll-forward) is matched by its tag.
            Some(target_version) => releases.iter().find(|r| &r.id == target_version),
            // Otherwise pick the newest release with an asset for this platform
            // that is newer than what we're running.
            None => Release::get_latest(releases.iter().filter(|&r| {
                r.get_variant().is_some()
                    && r.version > current_version
                    && (!r.prerelease || include_prerelease)
            })),
        };

        match target_release {
            Some(release) => {
                sentry::capture_message(
                    &format!("Starting Update to {}", release.id),
                    sentry::Level::Info,
                );
                writeln!(core.output(), "Downloading update {}...", &release.id)
                    .to_human_error()?;
                if manager.update(release).await? {
                    writeln!(
                        core.output(),
                        "Shutting down to complete the update operation."
                    )
                    .to_human_error()?;
                }
            }
            None if requested_version.is_some() => {
                return Err(human_errors::user(
                    "Could not find an available update for your platform matching the version you provided.",
                    &[
                        "If you would like to switch to a specific version, ensure that it is available by running `git-tool update --list`.",
                    ],
                ));
            }
            None => {
                writeln!(
                    core.output(),
                    "It doesn't look like there is an update available for your platform yet."
                )
                .to_human_error()?;
                writeln!(
                    core.output(),
                    "If you would like to rollback to a specific version, you can do so with `gt update v{}`.",
                    version!()
                ).to_human_error()?;
            }
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt update",
        skip(self, _core, completer, _matches)
    )]
    async fn complete(&self, _core: &Core, completer: &Completer, _matches: &ArgMatches) {
        if let Ok(releases) = crate::update::manager().get_releases().await {
            completer.offer_many(
                releases
                    .iter()
                    .filter(|r| r.get_variant().is_some())
                    .map(|r| &r.id),
            );
        }
    }
}

/// Render the `gt update --list` output: one line per release, prefixed with a
/// status marker — `*` for the version we're running, `!` for a release with no
/// asset for this platform, and a space otherwise — and a `(pre-release)` suffix
/// for pre-releases.
fn format_release_list(releases: &[Release], current_version: &semver::Version) -> String {
    use std::fmt::Write;

    let mut output = String::new();
    for release in releases {
        let style = if release.version == *current_version {
            "*"
        } else if release.get_variant().is_none() {
            "!"
        } else {
            " "
        };

        let suffix = if release.prerelease {
            " (pre-release)"
        } else {
            ""
        };

        let _ = writeln!(output, "{style} {}{suffix}", release.id);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::MockConsoleProvider;
    use std::sync::Arc;

    fn release(id: &str, version: &str, prerelease: bool, supported: bool) -> Release {
        Release {
            id: id.to_string(),
            changelog: String::new(),
            version: version.parse().unwrap(),
            prerelease,
            variant: supported.then(|| update_rs::ReleaseVariant {
                name: format!("git-tool-test-{id}"),
                sha256: None,
            }),
        }
    }

    #[test]
    fn format_release_list_marks_current_unsupported_and_available() {
        let current: semver::Version = "2.0.0".parse().unwrap();
        let releases = vec![
            release("v2.0.0", "2.0.0", false, true),
            release("v1.5.0", "1.5.0", false, false),
            release("v2.1.0", "2.1.0", true, true),
        ];

        let listing = format_release_list(&releases, &current);
        let lines: Vec<&str> = listing.lines().collect();

        assert_eq!(
            lines[0], "* v2.0.0",
            "the current version is marked with '*'"
        );
        assert_eq!(
            lines[1], "! v1.5.0",
            "a release with no asset for this platform is marked with '!'"
        );
        assert_eq!(
            lines[2], "  v2.1.0 (pre-release)",
            "an available pre-release is left unmarked and labelled"
        );
    }

    #[test]
    fn run_resume_bubbles_up_failures_without_reporting() {
        // `gt update --state <json>` resumes an in-progress update. The command
        // shouldn't report resume failures to Sentry itself; it bubbles them up so
        // `main` decides what to record (only system-caused errors are). Resuming
        // a cleanup phase with no temporary application path is a system error we
        // can trigger without any network access.
        let events = sentry::test::with_captured_events(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("we should be able to build a Tokio runtime");

            rt.block_on(async {
                let console = Arc::new(MockConsoleProvider::new());
                let core = Core::builder()
                    .with_default_config()
                    .with_console(console.clone())
                    .build();

                let cmd = UpdateCommand {};
                let args =
                    cmd.app()
                        .get_matches_from(vec!["update", "--state", r#"{"phase":"cleanup"}"#]);

                cmd.run(&core, &args)
                    .await
                    .expect_err("resuming a cleanup phase without a temporary path should fail");
            });
        });

        assert!(
            events.is_empty(),
            "the update command should not report resume failures to Sentry directly"
        );
    }
}
