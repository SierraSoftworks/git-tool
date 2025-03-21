use super::*;
use crate::core::Target;
use clap::Arg;
use tracing_batteries::prelude::*;
use crate::git;

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
            .arg(Arg::new("update-git-remote")
                .long("update-git-remote")
                .help("Also update the git remote URL after renaming.")
                .action(clap::ArgAction::SetTrue))
    }

    #[tracing::instrument(name = "gt rename", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, errors::Error> {
        let repo_name = matches.get_one::<String>("repo").ok_or_else(|| {
            errors::user(
                "No repository name was provided.",
                "Provide the name of the repository you wish to rename.",
            )
        })?;

        let new_name = matches.get_one::<String>("new_name").ok_or_else(|| {
            errors::user(
                "No repository name was provided.",
                "Provide the new name of the repository.",
            )
        })?;

        let update_remote = matches.get_flag("update-git-remote");

        let mut repo = core.resolver().get_best_repo(repo_name)?;
        if !repo.exists() {
            return Err(errors::user(
                "You didn't specify the repository you wanted to rename.",
                "Remember to specify a repository name in it's fully-qualified form like this: 'git-tool rename gh:sierrasoftworks/git-tool gt'.")
            )
        }

        if let Err(err) = repo.rename(new_name) {
            return Err(errors::user_with_internal(
                "Could not remove the repository directory due to an error.",
                "Make sure you have the correct permissions to remove the directory.",
                err,
            ));
        }

        if update_remote {
            git::git_remote_update_repo_name(&repo.get_path(), new_name).await?;
        }

        Ok(0)
    }

    #[tracing::instrument(
        name = "gt complete -- gt rename",
        skip(self, core, completer, _matches)
    )]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer("--update-git-remote");

        completer.offer_many(core.config().get_aliases().map(|(a, _)| a));
        completer.offer_many(core.config().get_apps().map(|a| a.get_name()));

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


