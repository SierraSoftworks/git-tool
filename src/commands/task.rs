use super::*;
use crate::engine::RepoConfig;
use crate::errors::HumanErrorResultExt;
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
            .about("runs a task defined in the current repository's 'git-tool.yml' file")
            .long_about("This command runs a named task defined in the current repository's 'git-tool.yml' configuration file. Run it from within a repository (`gt task <task>`), or with no arguments to list the tasks available in the current repository.

Tasks are only executed once you have confirmed that you trust the repository's configuration. The first time a repository's configuration is seen (or whenever it changes) you will be shown its contents and asked whether you trust it.")
            .arg(Arg::new("task")
                    .help("The name of the task to run.")
                    .index(1))
    }

    #[tracing::instrument(name = "gt task", err, skip(self, core, matches))]
    async fn run(&self, core: &Core, matches: &ArgMatches) -> Result<i32, human_errors::Error> {
        let repo = core.resolver().get_current_repo()?;
        let task_name = matches.get_one::<String>("task").cloned();

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
        if let Ok(repo) = core.resolver().get_current_repo()
            && let Ok(Some(config)) = RepoConfig::for_repo(&repo)
        {
            completer.offer_many(config.task_names().sorted());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;

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
    async fn list_tasks_in_current_repo() {
        let cmd = TaskCommand {};
        let args = cmd.app().get_matches_from(vec!["task"]);

        let temp = tempfile::tempdir().unwrap();
        let repo_path = temp.path().join("repo");
        write_repo_config(&repo_path, EXAMPLE_CONFIG);

        let console = crate::console::mock();
        let core = Core::builder()
            .with_config_for_dev_directory(temp.path())
            .with_console(console.clone())
            .with_mock_resolver(move |mock| {
                let repo_path = repo_path.clone();
                mock.expect_get_current_repo().returning(move || {
                    Ok(Repo::new("gh:sierrasoftworks/test-task", repo_path.clone()))
                });
            })
            .build();

        cmd.assert_run_successful(&core, &args).await;
        assert!(
            console.to_string().contains("build"),
            "the available tasks should be listed"
        );
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
