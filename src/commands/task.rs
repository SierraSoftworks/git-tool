use super::*;
use crate::engine::{Identifier, Repo, RepoConfig, Target};
use crate::errors::HumanErrorResultExt;
use crate::tasks::*;
use clap::Arg;
use itertools::Itertools;
use tracing_batteries::prelude::*;

pub struct TaskCommand;
crate::command!(TaskCommand);

#[async_trait]
impl CommandRunnable for TaskCommand {
    fn name(&self) -> String {
        String::from("task")
    }

    fn app(&self) -> clap::Command {
        clap::Command::new(self.name())
            .version("1.0")
            .visible_aliases(["t", "run"])
            .about("runs a task defined in a repository's 'git-tool.yml' file")
            .long_about("This command runs a named task defined in a repository's 'git-tool.yml' configuration file. You can run it from within a repository (`gt task <task>`) or against a specific repository (`gt task <repo> <task>`). If the repository has not been cloned yet, it will be cloned automatically before the task runs.

Tasks are only executed once you have confirmed that you trust the repository's configuration. The first time a repository's configuration is seen (or whenever it changes) you will be shown its contents and asked whether you trust it.")
            .arg(Arg::new("repo")
                    .help("The repository to run the task in, or the name of the task when run within a repository.")
                    .index(1))
            .arg(Arg::new("task")
                    .help("The name of the task to run.")
                    .index(2))
    }

    #[tracing::instrument(name = "gt task", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
        let first = matches.get_one::<String>("repo");
        let second = matches.get_one::<String>("task");

        let provided: Vec<&String> = [first, second].into_iter().flatten().collect();

        let current_repo = core.resolver().get_current_repo();

        let (repo, task_name): (Repo, Option<String>) = if let Ok(current) = current_repo {
            match provided.as_slice() {
                // gt task <repo> <task>
                [repo, task] => (self.resolve_repo(core, repo)?, Some((*task).clone())),
                // gt task <task> (within the current repository)
                [value] => (current, Some((*value).clone())),
                // gt task (list the tasks available in the current repository)
                [] => (current, None),
                _ => unreachable!("clap only provides up to two positional arguments"),
            }
        } else {
            match provided.as_slice() {
                [repo, task] => (self.resolve_repo(core, repo)?, Some((*task).clone())),
                [_value] => {
                    return Err(human_errors::user(
                        "You are not currently within a repository, so you need to specify both the repository and the task to run.",
                        &[
                            "Run the task like this: 'git-tool task github.com/sierrasoftworks/git-tool build'.",
                        ],
                    ));
                }
                [] => {
                    return Err(human_errors::user(
                        "You did not specify a repository or a task to run.",
                        &[
                            "Run the task like this: 'git-tool task github.com/sierrasoftworks/git-tool build'.",
                        ],
                    ));
                }
                _ => unreachable!("clap only provides up to two positional arguments"),
            }
        };

        // Ensure the repository has been cloned before we try to load its
        // configuration or run any tasks.
        if !repo.exists() {
            sequence![GitClone::default()]
                .apply_repo(core, &repo)
                .await?;
        }

        let config = RepoConfig::for_repo(&repo)?.ok_or_else(|| {
            human_errors::user(
                format!(
                    "The repository '{}' does not contain a 'git-tool.yml' configuration file, so it has no tasks to run.",
                    repo
                ),
                &["Add a 'git-tool.yml' file to the root of the repository to define tasks."],
            )
        })?;

        let task_name = match task_name {
            Some(name) => name,
            None => {
                let mut output = core.output();
                let names: Vec<&String> = config.task_names().sorted().collect();
                if names.is_empty() {
                    writeln!(output, "The repository '{repo}' does not define any tasks.")
                        .to_human_error()?;
                } else {
                    writeln!(output, "Available tasks for '{repo}':").to_human_error()?;
                    for name in names {
                        writeln!(output, "  {name}").to_human_error()?;
                    }
                }
                return Ok(0);
            }
        };

        let task = config.get_task(&task_name).ok_or_else(|| {
            let available: String = config.task_names().sorted().join(", ");
            let message = if available.is_empty() {
                format!(
                    "The repository '{repo}' does not define a task called '{task_name}', and it has no tasks defined at all."
                )
            } else {
                format!(
                    "The repository '{repo}' does not define a task called '{task_name}'. Available tasks are: {available}."
                )
            };
            human_errors::user(
                message,
                &["Check the 'git-tool.yml' file in the repository for the list of available tasks."],
            )
        })?;

        if !crate::commands::trust::ensure_trusted(core, &repo, &config).await? {
            return Ok(1);
        }

        let app = task.to_app(&task_name);
        let status = core.launcher().run(&app, &repo).await?;

        Ok(status)
    }

    #[tracing::instrument(name = "gt complete -- gt task", skip(self, core, completer, _matches))]
    async fn complete(&self, core: &Core, completer: &Completer, _matches: &ArgMatches) {
        completer.offer_aliases(core);
        completer.offer_repos(core);

        if let Ok(repo) = core.resolver().get_current_repo()
            && let Ok(Some(config)) = RepoConfig::for_repo(&repo)
        {
            completer.offer_many(config.task_names().sorted());
        }
    }
}

impl TaskCommand {
    fn resolve_repo(&self, core: &Core, value: &str) -> Result<Repo, human_errors::Error> {
        let identifier: Identifier = value.parse()?;
        core.resolver().get_best_repo(&identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
    use mockall::predicate::eq;

    fn write_repo_config(path: &std::path::Path, contents: &str) {
        std::fs::create_dir_all(path).unwrap();
        std::fs::write(path.join("git-tool.yml"), contents).unwrap();
    }

    const EXAMPLE_CONFIG: &str =
        "tasks:\n  build:\n    command: echo\n    args:\n      - building\n";

    #[tokio::test]
    async fn run_task_in_current_repo() {
        let cmd = TaskCommand {};
        let args = cmd.app().get_matches_from(vec!["task", "build"]);

        let temp = tempfile::tempdir().unwrap();
        let repo_path = temp.path().join("repo");
        write_repo_config(&repo_path, EXAMPLE_CONFIG);

        let repo = Repo::new("gh:sierrasoftworks/test-task", repo_path.clone());
        let trusted_repo = repo.to_string();
        let config = RepoConfig::for_repo(&repo).unwrap().unwrap();

        let cfg = Config::for_dev_directory(temp.path())
            .with_trusted_repo(trusted_repo, config.hash().unwrap());

        let core = Core::builder()
            .with_config(cfg)
            .with_null_console()
            .with_mock_resolver(move |mock| {
                let repo_path = repo_path.clone();
                mock.expect_get_current_repo().returning(move || {
                    Ok(Repo::new("gh:sierrasoftworks/test-task", repo_path.clone()))
                });
            })
            .with_mock_launcher(|mock| {
                mock.expect_run()
                    .times(1)
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        cmd.assert_run_successful(&core, &args).await;
    }

    #[tokio::test]
    async fn run_task_against_specified_repo() {
        let cmd = TaskCommand {};
        let args = cmd
            .app()
            .get_matches_from(vec!["task", "sierrasoftworks/test-task", "build"]);

        let temp = tempfile::tempdir().unwrap();
        let repo_path = temp.path().join("repo");
        write_repo_config(&repo_path, EXAMPLE_CONFIG);

        let repo = Repo::new("gh:sierrasoftworks/test-task", repo_path.clone());
        let trusted_repo = repo.to_string();
        let config = RepoConfig::for_repo(&repo).unwrap().unwrap();

        let cfg = Config::for_dev_directory(temp.path())
            .with_trusted_repo(trusted_repo, config.hash().unwrap());

        let identifier: Identifier = "sierrasoftworks/test-task".parse().unwrap();
        let core = Core::builder()
            .with_config(cfg)
            .with_null_console()
            .with_mock_resolver(move |mock| {
                let repo_path = repo_path.clone();
                mock.expect_get_current_repo()
                    .returning(|| Err(human_errors::user("not in a repo", &[])));
                mock.expect_get_best_repo()
                    .with(eq(identifier.clone()))
                    .returning(move |_| {
                        Ok(Repo::new("gh:sierrasoftworks/test-task", repo_path.clone()))
                    });
            })
            .with_mock_launcher(|mock| {
                mock.expect_run()
                    .times(1)
                    .returning(|_, _| Box::pin(async { Ok(0) }));
            })
            .build();

        cmd.assert_run_successful(&core, &args).await;
    }

    #[tokio::test]
    async fn run_unknown_task_errors() {
        let cmd = TaskCommand {};
        let args = cmd.app().get_matches_from(vec!["task", "missing"]);

        let temp = tempfile::tempdir().unwrap();
        let repo_path = temp.path().join("repo");
        write_repo_config(&repo_path, EXAMPLE_CONFIG);

        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_null_console()
            .with_mock_resolver(move |mock| {
                let repo_path = repo_path.clone();
                mock.expect_get_current_repo().returning(move || {
                    Ok(Repo::new("gh:sierrasoftworks/test-task", repo_path.clone()))
                });
            })
            .build();

        assert!(cmd.run(&core, &args).await.is_err());
    }
}
