use super::*;
use crate::update::{GitHubSource, Release, ReleaseVariant, UpdateManager};
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
                .help("State information used to resume an update operation.")
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
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, engine::Error>
where {
        let current_version: semver::Version = version!().parse().map_err(|err| errors::system_with_internal(
            "Could not parse the current application version into a SemVer version number.",
            "Please report this issue to us on GitHub and try updating manually by downloading the latest release from GitHub once the problem is resolved.",
            err))?;
        let manager: UpdateManager<GitHubSource> = UpdateManager::default();

        let resume_successful = match matches.get_one::<String>("state") {
            Some(state) => {
                debug!("Received update state: {}", state);
                sentry::configure_scope(|scope| {
                    scope.set_extra("state", json!(state));
                });

                let state: crate::update::UpdateState = serde_json::from_str(state).map_err(|e| errors::system_with_internal(
                    "Could not deserialize the update state blob.",
                    "Please report this issue to us on GitHub and use the manual update process until this problem is resolved.",
                    e))?;
                sentry::configure_scope(|scope| {
                    scope.set_extra("state", serde_json::to_value(&state).unwrap_or_default());
                    scope.set_transaction(Some(&format!("update/{}", state.phase)));
                });

                info!("Resuming update in phase {}", state.phase);
                manager.resume(&state).await.inspect_err(|e| {
                    sentry::capture_error(&e);
                })?
            }
            None => false,
        };

        if resume_successful {
            sentry::capture_message("Resumed Update", sentry::Level::Info);
            return Ok(0);
        }

        let releases = manager.get_releases(core).await?;
        let current_variant = ReleaseVariant::default();

        if matches.get_flag("list") {
            for release in releases {
                let style = if release.version == current_version {
                    "*"
                } else if release.get_variant(&current_variant).is_none() {
                    "!"
                } else {
                    " "
                };

                let suffix = if release.prerelease {
                    " (pre-release)"
                } else {
                    ""
                };

                writeln!(core.output(), "{} {}{}", style, release.id, suffix)?;
            }

            return Ok(0);
        }

        let include_prerelease = matches.get_flag("prerelease");
        let mut target_release = Release::get_latest(releases.iter().filter(|&r| {
            r.get_variant(&current_variant).is_some()
                && r.version > current_version
                && (!r.prerelease || include_prerelease)
        }));

        if let Some(target_version) = matches.get_one::<String>("target-version") {
            target_release = releases.iter().find(|r| &r.id == target_version);
        }

        match target_release {
            Some(release) => {
                sentry::capture_message(&format!("Starting Update to {}", release.id), sentry::Level::Info);
                writeln!(core.output(), "Downloading update {}...", &release.id)?;
                if manager.update(core, release).await? {
                    writeln!(core.output(), "Shutting down to complete the update operation.")?;
                }
            },
            None if matches.contains_id("version") => {
                return Err(errors::user(
                    "Could not find an available update for your platform matching the version you provided.",
                    "If you would like to switch to a specific version, ensure that it is available by running `git-tool update --list`."))
            },
            None => {
                writeln!(
                    core.output(),
                    "It doesn't look like there is an update available for your platform yet."
                )?;
                writeln!(core.output(), "If you would like to rollback to a specific version, you can do so with `gt update v{}`.", version!())?;
            }
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt update",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        let manager: UpdateManager<GitHubSource> = UpdateManager::default();

        if let Ok(releases) = manager.get_releases(core).await {
            let current_variant = ReleaseVariant::default();
            completer.offer_many(
                releases
                    .iter()
                    .filter(|&r| r.get_variant(&current_variant).is_some())
                    .map(|r| &r.id),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::console::MockConsoleProvider;
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run_list() {
        let console = Arc::new(MockConsoleProvider::new());
        let core = Core::builder()
            .with_default_config()
            .with_console(console.clone())
            .build();

        let cmd = UpdateCommand {};
        let args = cmd.app().get_matches_from(vec!["update", "--list"]);

        cmd.assert_run_successful(&core, &args).await;

        print!("{}", console);

        let mut has_version = false;
        console.to_string().split_terminator('\n').for_each(|line| {
            assert!(
                line.starts_with('*') || line.starts_with('!') || line.starts_with(' '),
                "the output should contain a list of versions prefixed with a status indicator"
            );

            assert!(
                line[1..].starts_with(" v"),
                "the output should contain a list of versions prefixed with 'v...'"
            );

            has_version = true;
        });

        assert!(has_version, "the output should contain a list of versions");
    }
}
