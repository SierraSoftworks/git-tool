use super::*;
use crate::engine::{Identifier, Target};
use crate::tasks::*;
use clap::Arg;
use tracing_batteries::prelude::*;

pub struct RenameCommand;
crate::command!(RenameCommand);

#[async_trait]
impl CommandRunnable for RenameCommand {
    fn name(&self) -> String {
        String::from("rename")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .about("renames a repository on your local machine")
            .long_about("This command will rename the specified repository on your local machine. It requires that the repository name be provided in fully-qualified form.")
            .visible_aliases(["mv"])
            .arg(Arg::new("repo")
                .help("The name of the repository to rename.")
                .long_help("The repository to be renamed in fully-qualified form.")
                .index(1)
                .required(true))
            .arg(Arg::new("new_name")
                .help("The new name of the repository.")
                .long_help("The new name of the repository must not be in fully-qualified form.")
                .index(2)
                .required(true))
            .arg(Arg::new("no-move-remote")
                .long("no-move-remote")
                .help("Do not rename the remote repository (on supported services).")
                .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt rename", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
        let no_move_remote = matches.get_flag("no-move-remote");
        let repo_name: Identifier = matches
            .get_one::<String>("repo")
            .ok_or_else(|| {
                human_errors::user("The repository name to be moved was not provided and cannot be moved as a result.", &["Make sure to provide the name of the repository you want to rename."])
            })?
            .parse()?;

        let new_name = repo_name.resolve(matches.get_one::<String>("new_name").ok_or_else(|| {
            human_errors::user(
                format!("The new repository name to rename your repository {} to was not provided and cannot be moved as a result.", repo_name),
                &["Make sure to provide the new name of the repository you want to rename."],
            )
        })?)?;

        let repo = core.resolver().get_best_repo(&repo_name)?;
        if !repo.exists() {
            return Err(human_errors::user(
                "Could not find the repository directory due to an error.",
                &[
                    "Make sure you have the correct permissions to rename the directory. Remember to specify a repository name in it's fully-qualified form like this: 'git-tool rename gh:sierrasoftworks/git-tool gt'.",
                ],
            ));
        }

        let new_repo = core.resolver().get_best_repo(&new_name)?;

        sequence![
            MoveDirectory {
                new_path: new_repo.path.clone(),
            },
            MoveRemote {
                enabled: !no_move_remote,
                target: new_repo.clone()
            }
        ]
        .apply_repo(core, &repo.clone())
        .await?;

        // Don't forget to update the remote URL to match the new repository name
        GitRemote { name: "origin" }
            .apply_repo(core, &new_repo)
            .await?;

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt rename",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--update-git-remote");

        completer.offer_many(core.config().get_aliases().map(|(a, _)| a));

        let default_svc = core
            .config()
            .get_default_service()
            .map(|s| s.name.clone())
            .unwrap_or_default();

        if let Ok(repos) = core.resolver().get_repos() {
            completer.offer_many(
                repos
                    .iter()
                    .filter(|r| r.service == default_svc)
                    .map(|r| r.get_full_name()),
            );
            completer.offer_many(
                repos
                    .iter()
                    .map(|r| format!("{}:{}", &r.service, r.get_full_name())),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MockHttpRoute;
    use mockall::predicate;
    use tempfile::tempdir;

    #[rstest::rstest]
    #[case::rename(
        "gh:git-fixtures/basic",
        "gh:git-fixtures/renamed",
        online::service::github::mocks::repo_update_name("git-fixtures/basic")
    )]
    #[case::transfer(
        "gh:git-fixtures/basic",
        "gh:fixtures/basic",
        online::service::github::mocks::repo_transfer("git-fixtures/basic")
    )]
    #[case::different_service("gh:git-fixtures/basic", "ghp:git-fixtures/basic", vec![])]
    #[tokio::test]
    async fn rename_repo(
        #[case] source_repo: &str,
        #[case] target_repo: &str,
        #[case] github_calls: Vec<MockHttpRoute>,
    ) {
        let cmd = RenameCommand {};

        let args = cmd
            .app()
            .get_matches_from(vec!["rename", source_repo, target_repo]);

        let temp = tempdir().unwrap();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_mock_http_client(github_calls)
            .with_mock_keychain(|c| {
                c.expect_get_token()
                    .with(predicate::eq("gh"))
                    .returning(|_| Ok("test_token".into()));
            })
            .with_null_console()
            .build();

        let src_repo = core
            .resolver()
            .get_best_repo(&source_repo.parse().unwrap())
            .unwrap();

        GitInit {}.apply_repo(&core, &src_repo).await.unwrap();

        assert!(src_repo.path.exists());
        assert!(src_repo.valid());

        cmd.assert_run_successful(&core, &args).await;

        assert!(
            !src_repo.path.exists(),
            "the old repo should not longer exist after being moved"
        );

        let new_repo = core
            .resolver()
            .get_best_repo(&target_repo.parse().unwrap())
            .unwrap();

        assert!(
            new_repo.path.exists(),
            "the repo should now exist at the new path"
        );
    }
}
