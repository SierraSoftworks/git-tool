use super::*;
use crate::core::Target;
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
            .arg(Arg::new("no-update-remote")
                .long("no-update-remote")
                .help("Also update the git remote URL after renaming.")
                .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt rename", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let no_update_remote = matches.get_flag("no-update-remote");
        let repo_name = matches.get_one::<String>("repo").ok_or_else(|| {
            errors::user(
                "The repository name to be moved was not provided and cannot be moved as a result.",
                "Make sure to provide the name of the repository you want to rename.",
            )
        })?;

        let new_name = matches.get_one::<String>("new_name").ok_or_else(|| {
            errors::user(
                format!("The new repository name to rename your repository {} to was not provided and cannot be moved as a result.", repo_name).as_str(),
                "Make sure to provide the new name of the repository you want to rename."
            )
        })?;

        let repo = core.resolver().get_best_repo(repo_name)?;
        if !repo.exists() {
            return Err(errors::user(
                "Could not find the repository directory due to an error.",
                "Make sure you have the correct permissions to rename the directory. Remember to specify a repository name in it's fully-qualified form like this: 'git-tool rename gh:sierrasoftworks/git-tool gt'.")
            );
        }

        let new_repo = core.resolver().get_best_repo(new_name)?;

        // Update the Git-Remote
        if !no_update_remote {
            GitMoveUpstream {
                new_repo: new_repo.clone(),
            }
            .apply_repo(core, &repo.clone())
            .await?;
        }

        let move_task = MoveDirectory {
            new_path: new_repo.path.clone(),
        };

        // Move the directory
        if let Err(err) = move_task.apply_repo(core, &repo).await {
            // Roll back the Git-remote change if needed
            if !no_update_remote {
                if let Err(rollback_err) =
                    (GitRemote { name: "origin" }).apply_repo(core, &repo).await
                {
                    tracing::warn!(
                        error = ?rollback_err,
                        "Failed to roll back Git remote change after move failure"
                    );
                }
            }

            return Err(err);
        }

        assert!(new_repo.valid());

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
    use crate::git::git_remote_get_url;
    use tempfile::tempdir;

    #[tokio::test]
    async fn rename_repo_update_upstream() {
        let cmd = RenameCommand {};

        let args = cmd.app().get_matches_from(vec![
            "rename",
            "gh:git-fixtures/basic",
            "gh:git-fixtures/renamed",
        ]);

        let temp = tempdir().unwrap();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        let repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/basic")
            .unwrap();

        GitClone {}.apply_repo(&core, &repo).await.unwrap();

        assert!(repo.path.exists());
        assert!(repo.valid());

        let remote = git_remote_get_url(&repo.path, "origin").await;
        assert!(remote.is_ok());

        let remote_list = remote.unwrap();
        assert_eq!(remote_list.len(), 1);
        let remote_url = remote_list.first().unwrap();
        assert!(
            remote_url.contains("git-fixtures/basic"),
            "Unexpected remote url: {remote_url}"
        );

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_eq!(status, 0, "the command should exit successfully");
            }
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            !repo.path.exists(),
            "the repo should be moved to the correct directory"
        );

        let new_repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/renamed")
            .unwrap();

        assert!(
            new_repo.path.exists(),
            "the repo should be moved to the correct directory"
        );

        let remote = git_remote_get_url(&new_repo.path, "origin").await;
        assert!(remote.is_ok());

        let remote_list = remote.unwrap();
        assert_eq!(remote_list.len(), 1);
        let remote_url = remote_list.first().unwrap();
        assert!(
            remote_url.contains("git-fixtures/renamed"),
            "Unexpected remote url: {remote_url}"
        );
    }

    #[tokio::test]
    async fn rename_repo_no_update_upstream() {
        let cmd = RenameCommand {};

        let args = cmd.app().get_matches_from(vec![
            "rename",
            "gh:git-fixtures/basic",
            "gh:git-fixtures/renamed",
            "--no-update-remote",
        ]);

        let temp = tempdir().unwrap();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        let repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/basic")
            .unwrap();

        GitClone {}.apply_repo(&core, &repo).await.unwrap();

        assert!(repo.path.exists());
        assert!(repo.valid());

        let remote = git_remote_get_url(&repo.path, "origin").await;
        assert!(remote.is_ok());

        let remote_list = remote.unwrap();
        assert_eq!(remote_list.len(), 1);
        let remote_url = remote_list.first().unwrap();
        assert!(
            remote_url.contains("git-fixtures/basic"),
            "Unexpected remote url: {remote_url}"
        );

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_eq!(status, 0, "the command should exit successfully");
            }
            Err(err) => panic!("{}", err.message()),
        }

        assert!(
            !repo.path.exists(),
            "the repo should be moved to the correct directory"
        );

        let new_repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/renamed")
            .unwrap();

        assert!(
            new_repo.path.exists(),
            "the repo should be moved to the correct directory"
        );

        let remote = git_remote_get_url(&new_repo.path, "origin").await;
        assert!(remote.is_ok());

        let remote_list = remote.unwrap();
        assert_eq!(remote_list.len(), 1);
        assert_ne!(
            remote_list.first().unwrap(),
            "git@github.com:git-fixtures/renamed.git"
        );

        let remote_url = remote_list.first().unwrap();
        assert!(
            remote_url.contains("git-fixtures/basic"),
            "Unexpected remote url: {remote_url}"
        );
    }

    #[tokio::test]
    async fn rename_folder_should_not_work() {
        let cmd = RenameCommand {};

        let args = cmd.app().get_matches_from(vec![
            "rename",
            "gh:git-fixtures/basic",
            "gh:fixtures/basic",
        ]);

        let temp = tempdir().unwrap();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .build();

        let repo = core
            .resolver()
            .get_best_repo("gh:git-fixtures/basic")
            .unwrap();

        GitClone {}.apply_repo(&core, &repo).await.unwrap();

        assert!(repo.path.exists());
        assert!(repo.valid());

        let remote = git_remote_get_url(&repo.path, "origin").await;
        assert!(remote.is_ok());

        let remote_list = remote.unwrap();
        assert_eq!(remote_list.len(), 1);
        let remote_url = remote_list.first().unwrap();
        assert!(
            remote_url.contains("git-fixtures/basic"),
            "Unexpected remote url: {remote_url}"
        );

        match cmd.run(&core, &args).await {
            Ok(status) => {
                assert_ne!(status, 0, "the command should not exit successfully");
            }
            Err(err) => {
                assert!(
                    err.message()
                        .contains("Could not rename the repository directory"),
                    "the command should not allow renaming the directory"
                )
            }
        }

        assert!(repo.path.exists(), "the repo should not be moved");

        let new_repo = core.resolver().get_best_repo("gh:fixtures/basic").unwrap();

        assert!(
            !new_repo.path.exists(),
            "the repo should be moved to the new directory, therefore the new directory should not exist"
        );

        let remote = git_remote_get_url(&repo.path, "origin").await;
        assert!(remote.is_ok());

        let remote_list = remote.unwrap();
        assert_eq!(remote_list.len(), 1);
        let remote_url = remote_list.first().unwrap();
        assert!(
            remote_url.contains("git-fixtures/basic"),
            "Unexpected remote url: {remote_url}"
        );
    }
}
