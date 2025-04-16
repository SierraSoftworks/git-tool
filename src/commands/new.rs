use super::*;
use crate::engine::Identifier;
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
            .long_about("Creates a new repository with the provided name.")
            .arg(
                Arg::new("repo")
                    .help("The name of the repository to create.")
                    .index(1),
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
    }

    #[tracing::instrument(name = "gt new", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo_id = matches
            .get_one::<String>("repo")
            .ok_or_else(|| {
                errors::user(
                    "No repository name provided for creation.",
                    "Please provide a repository name when calling this method: git-tool new my/repo",
                )
            })?
            .parse()?;

        let repo = core.resolver().get_best_repo(&repo_id)?;

        if repo.valid() {
            return Ok(0);
        }

        if let Some(from_repo) = matches.get_one::<String>("from") {
            let from_repo_id: Identifier = from_repo.as_str().parse()?;
            let from_repo = core.resolver().get_best_repo(&from_repo_id)?;
            let from_service = core.config().get_service(&from_repo.service)?;
            let from_url = from_service.get_git_url(&from_repo)?;

            let target_service = core.config().get_service(&from_repo.service)?;
            let target_url = target_service.get_git_url(&repo)?;

            let tasks = sequence![
                ForkRepository {
                    from_repo: from_repo.clone()
                },
                GitClone::with_path(Some(repo.path.clone())),
                GitAddRemote {
                    name: "origin".into(),
                    url: target_url,
                },
                GitAddRemote {
                    name: "upstream".into(),
                    url: from_url,
                }
            ];

            tasks.apply_repo(core, &from_repo).await?;
        } else {
            let tasks = sequence![
                EnsureNoRemote {
                    enabled: !matches.get_flag("no-check-exists")
                },
                GitInit {},
                GitRemote { name: "origin" },
                GitCheckout { branch: "main" },
                CreateRemote {
                    enabled: !matches.get_flag("no-create-remote")
                }
            ];

            tasks.apply_repo(core, &repo).await?;
        };

        if matches.get_flag("open") || core.config().get_features().has(features::OPEN_NEW_REPO) {
            let app = core.config().get_default_app().ok_or_else(|| errors::user(
                "No default application available.",
                "Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."))?;

            let status = core.launcher().run(app, &repo).await?;
            return Ok(status);
        }

        Ok(0)
    }

    #[tracing::instrument(name = "gt complete -- gt new", skip(self, core, completer, _matches))]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--open");
        completer.offer("--no-create-remote");
        completer.offer("--from");

        if let Ok(repos) = core.resolver().get_repos() {
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
    use mockall::predicate::eq;

    use super::*;

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

        let repo = core
            .resolver()
            .get_best_repo(&"gh:test/new-repo-partial".parse().unwrap())
            .unwrap();
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

        let repo = core
            .resolver()
            .get_best_repo(&"gh:test/new-repo-full".parse().unwrap())
            .unwrap();
        assert!(!repo.valid());

        cmd.assert_run_successful(&core, &args).await;

        assert!(repo.valid());
    }
}
