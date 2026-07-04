use super::*;
use crate::engine::Repo;
use crate::{engine::features, tasks::*};
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct NewCommand;
crate::command!(NewCommand);

#[async_trait]
impl CommandRunnable for NewCommand {
    fn name(&self) -> String {
        "new".into()
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .about("creates a new repository")
            .visible_aliases(["n", "create"])
            .long_about("Creates a new repository with the provided name.

You may also provide an application to open the repository with once it has been created, along with any number of KEY=VALUE tokens to override environment variables for that application (for example `gt new my/repo shell FOO=bar`). Providing an application implies `--open`.")
            .arg(
                Arg::new("args")
                    .help("The repository to create, an optional app to open it with, and any KEY=VALUE environment overrides (in any order).")
                    .action(clap::ArgAction::Append),
            )
            .arg(
                Arg::new("open")
                    .long("open")
                    .short('o')
                    .help("opens the repository in your default application after it is created.")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-create-remote")
                    .long("no-create-remote")
                    .short('R')
                    .help("prevent the creation of a remote repository (on supported services)")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-check-exists")
                    .long("no-check-exists")
                    .short('E')
                    .help("don't check whether the repository already exists on the remote service before creating a new local repository")
                    .action(clap::ArgAction::SetTrue),
            )
            .arg(
                Arg::new("from")
                    .long("from")
                    .alias("fork")
                    .short('f')
                    .help("create a fork of an existing remote (on supported services) or a copy of an existing remote repository (on unsupported services)"),
            )

            .arg(
                Arg::new("fork-all-branches")
                    .long("fork-all-branches")
                    .short('A')
                    .help("when forking from an existing repository, fork all branches (Default: False - default branch only).")
                    .action(clap::ArgAction::SetTrue),
            )
    }

    #[tracing::instrument(name = "gt new", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
        // `new` always creates an explicit target, so there is no implied
        // context; a lone token is therefore always treated as the repository.
        let parsed = crate::completion::parse::<Repo>(core, None, matches)?;
        let repo = &parsed.target;

        if repo.valid() {
            return Ok(0);
        }

        let tasks = if let Some(from_repo) = matches.get_one::<String>("from") {
            let from_repo: Repo = core.resolve(from_repo.as_str())?;
            let from_service = core.config().get_service(&from_repo.service)?;
            let from_url = from_service.get_git_url(&from_repo)?;

            let target_service = core.config().get_service(&from_repo.service)?;
            let target_url = target_service.get_git_url(repo)?;

            sequence![
                ForkRemote {
                    from_repo: from_repo.clone(),
                    default_branch_only: !matches.get_flag("fork-all-branches"),
                },
                GitClone::with_url(&from_url),
                GitAddRemote {
                    name: "origin".into(),
                    url: target_url,
                },
                GitAddRemote {
                    name: "upstream".into(),
                    url: from_url,
                }
            ]
        } else {
            sequence![
                EnsureNoRemote {
                    enabled: !matches.get_flag("no-check-exists")
                },
                GitInit {},
                GitRemote { name: "origin" },
                GitCheckout { branch: "main" },
                CreateRemote {
                    enabled: !matches.get_flag("no-create-remote")
                }
            ]
        };

        tasks.apply_repo(core, repo).await?;

        // Naming an application is a shorthand for `--open`: if the user told us
        // what to launch, they clearly want the repository opened.
        let should_open = parsed.app.is_some()
            || matches.get_flag("open")
            || core.config().get_features().has(features::OPEN_NEW_REPO);

        if should_open {
            let app = parsed.launch_app(core)?;
            let status = core.launcher().run(&app, repo).await?;
            return Ok(status);
        }

        Ok(0)
    }

    #[tracing::instrument(name = "gt complete -- gt new", skip(self, core, completer, _matches))]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--open");
        completer.offer("--no-create-remote");
        completer.offer("--from");

        let repos: Result<Vec<Repo>, _> = core.resolve_many(());
        if let Ok(repos) = repos {
            let mut namespaces = std::collections::HashSet::new();
            let default_svc = core
                .config()
                .get_default_service()
                .map(|s| s.name.clone())
                .unwrap_or_default();

            for repo in repos {
                if repo.service == default_svc {
                    namespaces.insert(format!("{}/", &repo.namespace));
                }

                namespaces.insert(format!("{}:{}/", &repo.service, &repo.namespace));
            }

            completer.offer_many(namespaces.iter().map(|s| s.as_str()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Identifier, Repo};
    use mockall::predicate::eq;
    use rstest::rstest;
    use tempfile::tempdir;

    #[tokio::test]
    async fn run_partial() {
        let cmd = NewCommand {};

        let args = cmd
            .app()
            .get_matches_from(vec!["new", "test/new-repo-partial"]);

        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("gh").join("test").join("new-repo-partial"))
            .expect("create test repo dir");

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_http_client(online::service::github::mocks::get_repo_not_exists(
                "test/new-repo-partial",
            ))
            .build();

        let repo: Repo = core.resolve("gh:test/new-repo-partial").unwrap();
        assert!(!repo.valid());
        cmd.assert_run_successful(&core, &args).await;

        assert!(repo.valid());
    }

    #[tokio::test]
    async fn run_fully_qualified() {
        let cmd = NewCommand {};

        let args = cmd
            .app()
            .get_matches_from(vec!["new", "gh:test/new-repo-full"]);

        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("gh").join("test").join("new-repo-full"))
            .expect("create test repo dir");

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_http_client(online::service::github::mocks::get_repo_not_exists(
                "test/new-repo-full",
            ))
            .build();

        let repo: Repo = core.resolve("gh:test/new-repo-full").unwrap();
        assert!(!repo.valid());

        cmd.assert_run_successful(&core, &args).await;

        assert!(repo.valid());
    }

    #[tokio::test]
    async fn run_with_env_override() {
        let cmd = NewCommand {};

        // An app token implies `--open`; the KEY=VALUE token becomes a literal
        // override on the launched app. `-R`/`-E` keep the creation local so the
        // test doesn't touch the network.
        let args = cmd.app().get_matches_from(vec![
            "new",
            "test/env-repo",
            "test-app",
            "FOO=bar",
            "-R",
            "-E",
        ]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = crate::engine::Config::from_str(&format!(
            "
directory: {}

apps:
  - name: test-app
    command: test
",
            temp.path().display(),
        ))
        .unwrap();

        let repo_path = temp.path().join("gh").join("test").join("env-repo");
        std::fs::create_dir_all(&repo_path).expect("create test repo dir");

        let core = Core::builder()
            .with_config(cfg)
            .with_mock_resolver(move |mock| {
                let repo_path = repo_path.clone();
                mock.expect_get_best_repo()
                    .returning(move |_| Ok(Repo::new("gh:test/env-repo", repo_path.clone())));
            })
            .with_mock_launcher(|mock| {
                mock.expect_run()
                    .once()
                    .withf(|app, _| {
                        app.get_name() == "test-app"
                            && app
                                .get_overrides()
                                .contains(&("FOO".to_string(), "bar".to_string()))
                    })
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        cmd.assert_run_successful(&core, &args).await;
    }

    #[rstest]
    #[cfg(feature = "auth")]
    #[case(
        "gh:git-fixtures/basic",
        "git-fixtures/basic",
        "gh:cedi/basic",
        "cedi/basic"
    )]
    #[case(
        "gh:git-fixtures/basic",
        "git-fixtures/basic",
        "gh:SierraSoftworks/basic",
        "SierraSoftworks/basic"
    )]
    #[tokio::test]
    #[cfg_attr(feature = "pure-tests", ignore)]
    async fn fork_repo(
        #[case] source_repo: &str,
        #[case] source: &str,
        #[case] target_repo: &str,
        #[case] target: &str,
    ) {
        let cmd = NewCommand {};

        let args = cmd
            .app()
            .get_matches_from(vec!["new", target_repo, "--fork", source_repo]);

        let temp = tempdir().unwrap();
        let temp_path = temp.path().to_path_buf();

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_http_client(crate::online::service::github::mocks::repo_fork(source))
            .with_mock_keychain(|mock| {
                mock.expect_get_token()
                    .with(eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_mock_resolver(|mock| {
                let source_temp_path = temp_path.clone();
                let source = source.to_owned();
                let source_segments = source.split('/');
                let full_source_path = source_segments
                    .fold(source_temp_path.clone(), |path, segment| path.join(segment));
                let source_identifier: Identifier = source_repo.parse().unwrap();
                mock.expect_get_best_repo()
                    .with(eq(source_identifier))
                    .times(1)
                    .returning(move |_| {
                        Ok(Repo::new("gh:git-fixtures/basic", full_source_path.clone()))
                    });

                let target_temp_path = temp_path.clone();
                let target = target.to_owned();
                let target_segments = target.split('/');
                let full_target_path = target_segments
                    .fold(target_temp_path.clone(), |path, segment| path.join(segment));
                let target_identifier: Identifier = target_repo.parse().unwrap();
                mock.expect_get_best_repo()
                    .with(eq(target_identifier))
                    .times(2)
                    .returning(move |_| {
                        Ok(Repo::new("gh:git-fixtures/empty", full_target_path.clone()))
                    });
            })
            .build();

        let repo: Repo = core.resolve(target_repo).unwrap();

        assert!(!repo.valid());

        cmd.assert_run_successful(&core, &args).await;

        assert!(repo.valid());
    }
}
