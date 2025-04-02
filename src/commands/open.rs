use super::*;
use crate::engine::features;
use crate::tasks::*;
use crate::update::{GitHubSource, Release, ReleaseVariant};
use crate::{engine::Target, update::UpdateManager};
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct OpenCommand;
crate::command!(OpenCommand);

#[async_trait]
impl CommandRunnable for OpenCommand {
    fn name(&self) -> String {
        String::from("open")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .visible_aliases(["o", "run"])
            .about("opens a repository using an application defined in your config")
            .long_about("This command launches an application defined in your configuration within the specified repository. You can specify any combination of alias, app and repo. Aliases take precedence over repos, which take precedence over apps. When specifying an app, it should appear before the repo/alias parameter. If you are already inside a repository, you can specify only an app and it will launch in the context of the current repo.
            
New applications can be configured either by making changes to your configuration, or by using the `git-tool config add` command to install them from the GitHub registry. For example, you can use `gt config add apps/bash` to configure `bash` as an available app.")
            .arg(Arg::new("app")
                    .help("The name of the application to launch.")
                    .index(1))
            .arg(Arg::new("repo")
                    .help("The name of the repository to open.")
                    .index(2))
            .arg(Arg::new("create")
                    .long("create")
                    .short('c')
                    .help("create the repository if it does not exist.")
                    .action(clap::ArgAction::SetTrue))
            .arg(Arg::new("no-create-remote")
                    .long("no-create-remote")
                    .short('R')
                    .help("prevent the creation of a remote repository (on supported services)")
                    .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt open", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        if core.config().get_config_file().is_none() {
            warn!("No configuration file has been loaded, continuing with defaults.");
            writeln!(core.output(),"Hi! It looks like you haven't set up a Git-Tool config file yet. Try running `git-tool setup` to get started or make sure you've set the GITTOOL_CONFIG environment variable.\n")?;
        }

        let (app, repo) = match helpers::get_launch_app(core, matches.get_one::<String>("app"), matches.get_one::<String>("repo"))? {
            helpers::LaunchTarget::AppAndTarget(app, target) => {
                (app, core.resolver().get_best_repo(&target)?)
            },
            helpers::LaunchTarget::App(app) => {
                (app, core.resolver().get_current_repo()?)
            },
            helpers::LaunchTarget::Target(target) => {
                let app = core.config().get_default_app().ok_or_else(|| errors::user(
                    "No default application available.",
                    "Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."))?;

                (app, core.resolver().get_best_repo(&target)?)
            },
            helpers::LaunchTarget::None => {
                return Err(errors::user(
                    "You did not specify the name of a repository to use.",
                    "Remember to specify a repository name like this: 'git-tool open github.com/sierrasoftworks/git-tool'."))
            }
        };

        if !repo.exists() {
            match sequence![GitClone {}].apply_repo(core, &repo).await {
                Ok(()) => {}
                Err(_) if matches.get_flag("create") => {
                    sequence![
                        GitInit {},
                        GitRemote { name: "origin" },
                        GitCheckout { branch: "main" },
                        CreateRemote {
                            enabled: !matches.get_flag("no-create-remote")
                        }
                    ]
                    .apply_repo(core, &repo)
                    .await?;
                }
                Err(e) => return Err(e),
            }
        }

        if core
            .config()
            .get_features()
            .has(features::CHECK_FOR_UPDATES)
        {
            let (status, latest_release) =
                futures::join!(core.launcher().run(app, &repo), self.check_for_update(core));

            if let Ok(Some(latest_release)) = latest_release {
                writeln!(core.output(), "A new version of Git-Tool is available (v{}). You can update it using `gt update`,", latest_release.version)?;
            }

            Ok(status?)
        } else {
            Ok(core.launcher().run(app, &repo).await?)
        }
    }

    #[tracing::instrument(
        name = "gt complete -- gt complete",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_aliases(core);
        completer.offer("--create");
        completer.offer("--no-create-remote");
        completer.offer_apps(core);
        completer.offer_repos(core);
    }
}

impl OpenCommand {
    #[tracing::instrument(err, skip(self, core))]
    async fn check_for_update(&self, core: &Core) -> Result<Option<Release>, errors::Error> {
        let current_version: semver::Version = version!().parse().map_err(|err| errors::system_with_internal(
            "Could not parse the current application version into a SemVer version number.",
            "Please report this issue to us on GitHub and try updating manually by downloading the latest release from GitHub once the problem is resolved.",
            err))?;

        info!("Current application version is v{}", current_version);

        let manager: UpdateManager<GitHubSource> = Default::default();

        let releases = manager.get_releases(core).await?;
        let current_variant = ReleaseVariant::default();

        let target_release = Release::get_latest(releases.iter().filter(|&r| {
            r.get_variant(&current_variant).is_some()
                && r.version > current_version
                && (!r.prerelease)
        }));

        Ok(target_release.cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
    use mockall::predicate::eq;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn run() {
        let cmd = OpenCommand {};

        let args = cmd.app().get_matches_from(vec!["open", "test-app", "repo"]);

        let cfg = Config::from_str(
            "
directory: /dev

apps:
  - name: test-app
    command: test
    args:
        - '{{ .Target.Name }}'

features:
  http_transport: true
",
        )
        .unwrap();

        let temp = tempdir().unwrap();
        std::fs::create_dir(temp.path().join("repo")).expect("create test repo dir");
        let temp_path = temp.path().to_owned();
        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(|mock| {
                let temp_path = temp_path.clone();
                let identifier: Identifier = "repo".parse().unwrap();
                mock.expect_get_best_repo()
                    .with(eq(identifier))
                    .returning(move |_| {
                        Ok(Repo::new("gh:git-fixtures/basic", temp_path.join("repo")))
                    });
            })
            .with_mock_launcher(|mock| {
                mock.expect_run()
                    .withf(|app, repo| app.get_name() == "test-app" && repo.get_name() == "basic")
                    .returning(|_, _| Box::pin(async { Ok(5) }));
            })
            .build();

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_eq!(
                    status, 5,
                    "the status code of the child app should be forwarded"
                );
            }
            Err(err) => panic!("{}", err.message()),
        }
    }
}
