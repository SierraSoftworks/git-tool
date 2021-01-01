use super::*;
use crate::update::{GitHubSource, Release, UpdateManager};
use clap::Arg;

pub struct UpdateCommand {}

impl Command for UpdateCommand {
    fn name(&self) -> String {
        String::from("update")
    }
    fn app<'a>(&self) -> clap::App<'a> {
        App::new(&self.name())
            .version("1.0")
            .about("updates Git-Tool automatically by fetching the latest release from GitHub")
            .long_about("Allows you to update Git-Tool to the latest version, or a specific version, automatically.")
            .arg(Arg::new("state")
                .long("state")
                .about("State information used to resume an update operation.")
                .hidden(true)
                .takes_value(true))
            .arg(Arg::new("list")
                .long("list")
                .about("Prints the list of available releases."))
            .arg(Arg::new("version")
                .about("The version you wish to update to. Defaults to the latest available version.")
                .index(1))
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for UpdateCommand {
    async fn run(&self, core: &C, matches: &clap::ArgMatches) -> Result<i32, crate::core::Error>
    where
        C: Core,
    {
        let mut output = core.output();

        let current_version: semver::Version = version!().parse().map_err(|err| errors::system_with_internal(
            "Could not parse the current application version into a SemVer version number.",
            "Please report this issue to us on GitHub and try updating manually by downloading the latest release from GitHub once the problem is resolved.",
            err))?;
        let manager: UpdateManager<C, GitHubSource> = UpdateManager::default();

        let resume_successful = match matches.value_of("state") {
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
                    scope.set_transaction(Some(&format!("update/{}", state.phase.to_string())));
                });

                info!("Resuming update in phase {}", state.phase.to_string());
                manager.resume(&state).await.map_err(|e| {
                    sentry::capture_error(&e);

                    e
                })?
            }
            None => false,
        };

        if resume_successful {
            sentry::capture_message("Resumed Update", sentry::Level::Info);
            return Ok(0);
        }

        let releases = manager.get_releases(core).await?;

        if matches.is_present("list") {
            for release in releases {
                let mut style = " ";
                if release.version == current_version {
                    style = "*";
                }

                writeln!(output, "{} {}", style, release.id)?;
            }

            return Ok(0);
        }

        let mut target_release =
            Release::get_latest(releases.iter().filter(|r| r.version > current_version));

        if let Some(target_version) = matches.value_of("version") {
            target_release = releases.iter().find(|r| r.id == target_version);
        }

        match target_release {
            Some(release) => {
                sentry::capture_message(&format!("Starting Update to {}", release.id), sentry::Level::Info);
                writeln!(output, "Downloading update {}...", &release.id)?;
                if manager.update(core, &release).await? {
                    writeln!(output, "Shutting down to complete the update operation.")?;
                }
            },
            None => {
                return Err(errors::user(
                    "Could not find an available update which matches your update criteria.",
                    "If you would like to switch to a specific version, ensure that it is available by running `git-tool update --list`."))
            }
        }

        Ok(0)
    }

    async fn complete(&self, core: &C, completer: &Completer, _matches: &ArgMatches) {
        let manager: UpdateManager<C, GitHubSource> = UpdateManager::default();

        match manager.get_releases(core).await {
            Ok(releases) => {
                completer.offer_many(releases.iter().map(|r| &r.id));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Config, CoreBuilder};
    use super::*;

    #[tokio::test]
    async fn run_list() {
        let cfg = Config::default();
        let core = CoreBuilder::default().with_config(&cfg).build();

        let output = crate::console::output::mock();

        let cmd = UpdateCommand {};
        let args = cmd.app().get_matches_from(vec!["update", "--list"]);

        match cmd.run(&core, &args).await {
            Ok(_) => {}
            Err(err) => panic!(err.message()),
        }

        assert!(
            output.to_string().contains("  v1.5.6\n"),
            "the output should contain a list of versions"
        );
    }
}
